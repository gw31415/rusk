use std::{
    collections::HashMap, future::Future, future::IntoFuture, path::PathBuf, pin::Pin, rc::Rc,
};

use deno_task_shell::{parser::SequentialList, ShellPipeReader, ShellPipeWriter, ShellState};
use futures::future::try_join_all;

use crate::digraph::{DigraphItem, TreeNode, TreeNodeCreationError};

/// Rusk error
#[derive(Debug, thiserror::Error)]
pub enum RuskError {
    /// TreeNode creation error
    #[error(transparent)]
    TreeNodeCreation(#[from] TreeNodeCreationError),
    /// Configuration parsing error
    #[error(transparent)]
    ConfigParse(#[from] ConfigParseError),
    /// Task execution error
    #[error(transparent)]
    Task(#[from] TaskError),
}

#[derive(Clone)]
pub struct IOSet {
    pub stdin: ShellPipeReader,
    pub stdout: ShellPipeWriter,
    pub stderr: ShellPipeWriter,
}

impl Default for IOSet {
    fn default() -> Self {
        Self {
            stdin: ShellPipeReader::stdin(),
            stdout: ShellPipeWriter::stdout(),
            stderr: ShellPipeWriter::stderr(),
        }
    }
}

/// Rusk configuration
pub struct Rusk {
    /// Tasks to be executed
    pub tasks: HashMap<String, Task>,
    /// Environment variables that are shared among all tasks
    pub envs: HashMap<String, String>,
}

async fn exec<T, E, I: IntoFuture<Output = Result<T, E>>>(node: TreeNode<I>) -> Result<T, E> {
    let TreeNode { item, mut children } = node;
    while !children.is_empty() {
        let mut buf = Vec::new();
        let mut tasks = Vec::new();
        for child in children {
            match Rc::try_unwrap(child) {
                Ok(node) => {
                    tasks.push(exec(node));
                }
                Err(rc) => {
                    buf.push(rc);
                }
            }
        }
        try_join_all(tasks).await?;
        children = buf;
    }
    item.await
}

impl Rusk {
    /// Execute tasks
    pub async fn exec(self, tasknames: &[String], io: IOSet) -> Result<(), RuskError> {
        let Rusk {
            tasks,
            envs: global_env,
        } = self;
        let executables = make_executable(tasks, io, global_env)?;
        let graph = TreeNode::new_vec(executables, tasknames)?;
        try_join_all(graph.into_iter().map(exec)).await?;
        Ok(())
    }
}

/// Task configuration
pub struct Task {
    /// Environment variables that are specific to this task
    pub envs: HashMap<String, String>,
    /// Script to be executed
    pub script: Option<String>,
    /// Working directory
    pub cwd: PathBuf,
    /// Dependencies
    pub depends: Vec<String>,
}

fn make_executable(
    tasks: HashMap<String, Task>,
    io: IOSet,
    global_env: HashMap<String, String>,
) -> Result<HashMap<String, TaskExecutable>, ConfigParseError> {
    let mut parsed_tasks: HashMap<String, TaskExecutable> = HashMap::new();

    for (task_name, task) in tasks {
        let script = {
            let mut items = Vec::new();
            if let Some(script) = task.script {
                for line in script.lines() {
                    items.extend(match deno_task_shell::parser::parse(line) {
                        Ok(script) => script.items,
                        Err(error) => {
                            return Err(ConfigParseError::ScriptParseError { task_name, error })?;
                        }
                    });
                }
            };
            SequentialList { items }
        };

        let Task {
            envs, cwd, depends, ..
        } = task;

        let Ok(cwd) = cwd.canonicalize() else {
            return Err(ConfigParseError::DirectoryNotFound(cwd));
        };

        parsed_tasks.insert(
            task_name.clone(),
            TaskExecutable {
                io: io.clone(),
                task_name,
                script,
                depends,
                envs: global_env.clone().into_iter().chain(envs).collect(),
                cwd,
            },
        );
    }

    Ok(parsed_tasks)
}

struct TaskExecutable {
    io: IOSet,
    task_name: String,
    envs: HashMap<String, String>,
    script: SequentialList,
    cwd: PathBuf,
    depends: Vec<String>,
}

impl IntoFuture for TaskExecutable {
    type Output = Result<(), TaskError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let Self {
                io,
                task_name,
                envs,
                script,
                cwd,
                ..
            } = self;
            let exit_code = deno_task_shell::execute_with_pipes(
                script,
                ShellState::new(envs, &cwd, Default::default(), Default::default()),
                io.stdin,
                io.stdout,
                io.stderr,
            )
            .await;
            if exit_code == 0 {
                Ok(())
            } else {
                Err(TaskError {
                    task_name,
                    exit_code,
                })
            }
        })
    }
}

impl DigraphItem for TaskExecutable {
    fn dependencies(&self) -> impl IntoIterator<Item: AsRef<str>> {
        self.depends.iter()
    }
}

/// Configuration parsing error
#[derive(Debug, thiserror::Error)]
pub enum ConfigParseError {
    /// Directory not found
    #[error("Directory not found: {0:?}")]
    DirectoryNotFound(PathBuf),
    /// Task script parse error
    #[error("Task {task_name:?} script parse error: {error:?}")]
    ScriptParseError {
        task_name: String,
        error: anyhow::Error,
    },
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("Task {task_name:?} failed with exit code {exit_code}")]
pub struct TaskError {
    pub task_name: String,
    pub exit_code: i32,
}
