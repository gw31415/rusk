use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    future::{Future, IntoFuture},
    ops::Deref,
    pin::Pin,
};

use deno_task_shell::{parser::SequentialList, ShellPipeReader, ShellPipeWriter, ShellState};
use futures::future::try_join_all;
use hashbrown::HashMap;
use tokio::sync::watch::Receiver;

use crate::{
    digraph::{DigraphItem, TreeNode, TreeNodeCreationError},
    fs::{RuskfileComposer, RuskfileConvertError},
    path::{get_current_dir, NormarizedPath},
    taskkey::{TaskKey, TaskKeyParseError, TaskKeyRelative},
};

type TaskTree = TreeNode<TaskKey, TaskExecutable>;

/// Rusk error
#[derive(Debug, thiserror::Error)]
pub enum RuskError {
    /// Task key parsing error
    #[error(transparent)]
    InvalidTaskKey(#[from] TaskKeyParseError),
    /// TreeNode creation error
    #[error(transparent)]
    TreeNodeBroken(#[from] TreeNodeCreationError<TaskKey>),
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
    tasks: HashMap<TaskKey, Task>,
}

impl TryFrom<RuskfileComposer> for Rusk {
    type Error = RuskfileConvertError;
    fn try_from(value: RuskfileComposer) -> Result<Self, Self::Error> {
        Ok(Rusk {
            tasks: value.try_into()?,
        })
    }
}

impl Rusk {
    /// Execute tasks
    pub async fn exec(
        self,
        args: impl IntoIterator<Item = String>,
        opts: ExecuteOpts,
    ) -> Result<(), RuskError> {
        let Rusk { tasks } = self;
        let tasks = into_executable(tasks, opts)?;
        let tk = args
            .into_iter()
            .map({
                fn f(s: String) -> Result<TaskKey, TaskKeyParseError> {
                    let key = TaskKeyRelative::try_from(s)?;
                    Ok(key.into_task_key(get_current_dir()))
                }
                f
            })
            .collect::<Result<Vec<_>, _>>()?;
        let graph = TreeNode::new_vec(tasks, tk)?;
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
    pub cwd: NormarizedPath,
    /// Dependencies
    pub depends: Vec<TaskKey>,
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

/// Alternative for `TryInto<HashMap<_, TaskExecutable>>` for `HashMap<_, Task>`
fn into_executable(
    tasks: HashMap<TaskKey, Task>,
    ExecuteOpts {
        envs: global_env,
        io,
    }: ExecuteOpts,
) -> Result<HashMap<TaskKey, TaskExecutable>, TaskParseError> {
    let mut parsed_tasks: HashMap<TaskKey, TaskExecutable> = HashMap::new();

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

        if !cwd.is_dir() {
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

async fn exec_all(roots: impl IntoIterator<Item = TaskTree>) -> TaskResult {
    async fn exec_node(node: &TaskTree) -> TaskResult {
        let child_futures = node.children.iter().map(|child| exec_node(child));
        try_join_all(child_futures).await?;
        node.item.as_future().await
    }

    let futures = roots
        .into_iter()
        .map(|root| async move { exec_node(&root).await });
    try_join_all(futures).await?;
    Ok(())
}

enum TaskExecutableState {
    Initialized(TaskExecutableInner),
    Processing(Receiver<Option<TaskResult>>),
    Done(TaskResult),
}

struct TaskExecutable(RefCell<TaskExecutableState>);

impl TaskExecutable {
    pub async fn as_future(&self) -> TaskResult {
        let res = 'res: {
            'early_return: {
                let mut rx = match &self.0.try_borrow().unwrap() as &TaskExecutableState {
                    TaskExecutableState::Done(result) => return result.clone(),
                    TaskExecutableState::Processing(rx) => {
                        if let Some(res) = rx.borrow().as_ref() {
                            break 'res res.clone();
                        }
                        rx.clone() // チャンネルをブロック外に持ち出し、**self.0 の参照を解放** する
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
                &mut self.0.try_borrow_mut().unwrap() as &mut TaskExecutableState,
                TaskExecutableState::Processing(rx),
            ) else {
                unreachable!()
            };
            let res = inner.into_future().await;
            tx.send(Some(res.clone())).unwrap();
            res
        };

        *self.0.try_borrow_mut().unwrap() = TaskExecutableState::Done(res.clone());
        res
    }
}

struct TaskExecutableInner {
    io: IOSet,
    task_name: TaskKey,
    envs: std::collections::HashMap<String, String>,
    script: SequentialList,
    cwd: NormarizedPath,
    depends: Vec<TaskKey>, // 依存関係の検索についてはTaskKeyを用いるか検討が必要
}

impl From<TaskExecutableInner> for TaskExecutable {
    fn from(val: TaskExecutableInner) -> Self {
        TaskExecutable(RefCell::new(TaskExecutableState::Initialized(val)))
    }
}

impl IntoFuture for TaskExecutableInner {
    type Output = TaskResult;
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

impl DigraphItem<TaskKey> for TaskExecutable {
    fn children(&self) -> impl Deref<Target = [TaskKey]> {
        Ref::map::<[TaskKey], _>(self.0.borrow(), |state| match state {
            TaskExecutableState::Initialized(inner) => inner.depends.as_slice(),
            _ => panic!("TaskExecutable is already called"),
        })
    }
}

/// Task parsing error
#[derive(Debug, thiserror::Error)]
pub enum TaskParseError {
    /// Directory not found
    #[error("Directory not found: {0}")]
    DirectoryNotFound(NormarizedPath),
    /// Task script parse error
    #[error("Task {task_name:?} script parse error: {error:?}")]
    ScriptParseError {
        task_name: TaskKey,
        error: anyhow::Error,
    },
}

/// Task execution error
#[derive(Debug, Clone, thiserror::Error)]
#[error("Task {task_name:?} failed with exit code {exit_code}")]
pub struct TaskError {
    pub task_name: TaskKey,
    pub exit_code: i32,
}

/// Task result alias
type TaskResult = Result<(), TaskError>;
