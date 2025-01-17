use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    future::{Future, IntoFuture},
    path::PathBuf,
    pin::Pin,
};

use deno_task_shell::{parser::SequentialList, ShellPipeReader, ShellPipeWriter, ShellState};
use futures::future::try_join_all;
use tokio::sync::watch::Receiver;

use crate::{
    digraph::{DigraphItem, TreeNode, TreeNodeCreationError},
    ruskfile::RuskfileComposer,
};

/// Rusk error
#[derive(Debug, thiserror::Error)]
pub enum RuskError {
    /// TreeNode creation error
    #[error(transparent)]
    TreeNodeBroken(#[from] TreeNodeCreationError),
    /// Task parsing error
    #[error(transparent)]
    TaskUnparsable(#[from] TaskParseError),
    /// Task execution error
    #[error(transparent)]
    TaskFailed(#[from] TaskError),
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
    tasks: HashMap<String, Task>,
}

impl From<RuskfileComposer> for Rusk {
    fn from(composer: RuskfileComposer) -> Self {
        Rusk {
            tasks: composer.into(),
        }
    }
}

impl Rusk {
    /// Execute tasks
    pub async fn exec(
        self,
        tasknames: impl IntoIterator<Item = String>,
        opts: ExecuteOpts,
    ) -> Result<(), RuskError> {
        let Rusk { tasks } = self;
        let executables = make_executable(tasks, opts)?;
        let graph = TreeNode::new_vec(executables, tasknames)?;
        exec_all(graph).await?;
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

/// Task execution global options
pub struct ExecuteOpts {
    /// Environment variables
    pub envs: HashMap<String, String>,
    /// IO
    pub io: IOSet,
}

impl Default for ExecuteOpts {
    fn default() -> Self {
        Self {
            envs: std::env::vars().collect(),
            io: Default::default(),
        }
    }
}

fn make_executable(
    tasks: HashMap<String, Task>,
    ExecuteOpts {
        envs: global_env,
        io,
    }: ExecuteOpts,
) -> Result<HashMap<String, TaskExecutable>, TaskParseError> {
    let mut parsed_tasks: HashMap<String, TaskExecutable> = HashMap::new();

    for (task_name, task) in tasks {
        let script = {
            let mut items = Vec::new();
            if let Some(script) = task.script {
                for line in script.lines() {
                    items.extend(match deno_task_shell::parser::parse(line) {
                        Ok(script) => script.items,
                        Err(error) => {
                            return Err(TaskParseError::ScriptParseError { task_name, error })?;
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
            return Err(TaskParseError::DirectoryNotFound(cwd));
        };
        if cwd.is_file() {
            return Err(TaskParseError::DirectoryNotFound(cwd));
        }

        parsed_tasks.insert(
            task_name.clone(),
            TaskExecutableInner {
                io: io.clone(),
                task_name,
                script,
                depends,
                envs: global_env.clone().into_iter().chain(envs).collect(),
                cwd,
            }
            .into(),
        );
    }

    Ok(parsed_tasks)
}

async fn exec_all(
    roots: impl IntoIterator<Item = TreeNode<TaskExecutable>>,
) -> Result<(), TaskError> {
    async fn exec_node(node: &TreeNode<TaskExecutable>) -> Result<(), TaskError> {
        let child_futures = node.children.iter().map(|child| exec_node(child));
        try_join_all(child_futures).await?;
        let res = 'res: {
            'early_return: {
                let mut rx = match &node.item.0.try_borrow().unwrap() as &TaskExecutableState {
                    TaskExecutableState::Done(result) => return result.clone(),
                    TaskExecutableState::Processing(rx) => {
                        if let Some(res) = rx.borrow().as_ref() {
                            break 'res res.clone();
                        }
                        rx.clone() // チャンネルをブロック外に持ち出し、**node.item.0 の参照を解放** する
                    }
                    _ => {
                        break 'early_return; // タスクを実行する必要がある
                    }
                };

                // タスクが実行中の場合 (Processing)、結果を待つ
                rx.changed().await.unwrap();
                break 'res rx.borrow().as_ref().unwrap().clone();
            }

            // もしタスクを実際に実行する場合、Watcherを作成して終了時に結果を送信する
            let (tx, rx) = tokio::sync::watch::channel(None);
            let TaskExecutableState::Initialized(inner) = std::mem::replace(
                &mut node.item.0.try_borrow_mut().unwrap() as &mut TaskExecutableState,
                TaskExecutableState::Processing(rx),
            ) else {
                unreachable!()
            };
            let res = inner.into_future().await;
            tx.send(Some(res.clone())).unwrap();
            res
        };

        *node.item.0.try_borrow_mut().unwrap() = TaskExecutableState::Done(res.clone());
        res
    }

    let futures = roots
        .into_iter()
        .map(|root| async move { exec_node(&root).await });
    try_join_all(futures).await?;
    Ok(())
}

enum TaskExecutableState {
    Initialized(TaskExecutableInner),
    Processing(Receiver<Option<Result<(), TaskError>>>),
    Done(Result<(), TaskError>),
}

struct TaskExecutable(RefCell<TaskExecutableState>);

struct TaskExecutableInner {
    io: IOSet,
    task_name: String,
    envs: HashMap<String, String>,
    script: SequentialList,
    cwd: PathBuf,
    depends: Vec<String>,
}

impl From<TaskExecutableInner> for TaskExecutable {
    fn from(val: TaskExecutableInner) -> Self {
        TaskExecutable(RefCell::new(TaskExecutableState::Initialized(val)))
    }
}

impl IntoFuture for TaskExecutableInner {
    type Output = Result<(), TaskError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let TaskExecutableInner {
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
        // TODO: Mutexの中身をコピーせずに参照を返す方法があればそれを使いたい
        let state: &TaskExecutableState = &self.0.borrow();
        match state {
            TaskExecutableState::Initialized(inner) => inner.depends.clone(),
            _ => panic!("TaskExecutable is already called"),
        }
    }
}

/// Task parsing error
#[derive(Debug, thiserror::Error)]
pub enum TaskParseError {
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

#[derive(Debug, Clone, thiserror::Error)]
#[error("Task {task_name:?} execution failed with exit code {exit_code}")]
pub struct TaskError {
    pub task_name: String,
    pub exit_code: i32,
}
