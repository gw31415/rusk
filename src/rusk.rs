use std::{collections::HashMap, path::PathBuf};

use deno_task_shell::{parser::SequentialList, ShellPipeReader, ShellPipeWriter, ShellState};
use futures::future::try_join_all;
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// Rusk error
#[derive(Debug, thiserror::Error)]
pub enum RuskError {
    /// Task execution error
    #[error(transparent)]
    TaskError(#[from] TaskError),
    /// JobSet creation error
    #[error(transparent)]
    JobSetCreationError(#[from] JobSetCreationError),
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

#[derive(Debug, thiserror::Error)]
pub enum JobSetCreationError {
    #[error(transparent)]
    ConfigParseError(#[from] ConfigParseError),
    #[error("Task {task_name:?} not found")]
    TaskNotFound { task_name: String },
}

/// Rusk configuration
pub struct Rusk {
    /// Tasks to be executed
    pub tasks: HashMap<String, Task>,
    /// Environment variables that are shared among all tasks
    pub envs: HashMap<String, String>,
}

impl Rusk {
    /// Execute tasks
    pub async fn execute(self, tasknames: &[String], io: IOSet) -> Result<(), RuskError> {
        let jobs = self.create_jobset(tasknames)?;
        let fs = jobs.jobs.into_iter().map(|task| task.execute(io.clone()));
        try_join_all(fs).await?;
        Ok(())
    }

    fn create_jobset(self, tasknames: &[String]) -> Result<JobSet, JobSetCreationError> {
        let Rusk {
            mut tasks,
            envs: global_env,
        } = self;
        let mut tasks: Vec<_> = if tasknames.is_empty() {
            tasks.into_iter().collect()
        } else {
            let mut res: HashMap<String, Task> = HashMap::new();
            let mut refer: Vec<&str> = tasknames.iter().map(AsRef::as_ref).collect();
            while let Some(name) = refer.pop() {
                if !res.contains_key(name) {
                    let name = name.to_string();
                    let Some(task) = tasks.remove(&name) else {
                        return Err(JobSetCreationError::TaskNotFound { task_name: name })?;
                    };
                    res.insert(name.clone(), task);
                }
            }
            res.into_iter().collect()
        };
        tasks.sort_by(|(_, t1), (_, t2)| t1.depends.len().cmp(&t2.depends.len()));

        let mut parsed_tasks: HashMap<String, Job> = HashMap::new();

        for (task_name, task) in tasks {
            if parsed_tasks.contains_key(&task_name) {
                return Err(ConfigParseError::DuplicateTaskName { task_name })?;
            }

            let script = {
                let mut items = Vec::new();
                for line in task.script.lines() {
                    items.extend(match deno_task_shell::parser::parse(line) {
                        Ok(script) => script.items,
                        Err(error) => {
                            return Err(ConfigParseError::ScriptParseError { task_name, error })?;
                        }
                    });
                }
                SequentialList { items }
            };

            let mut depends = Vec::new();
            for dep_name in &task.depends {
                let (tx, rx) = channel::<Result<(), ()>>(1);
                depends.push(rx);
                if let Some(dep_task) = parsed_tasks.get_mut(dep_name) {
                    dep_task.nexts.push(tx);
                } else {
                    return Err(ConfigParseError::UnexecutableTask { task_name })?;
                }
            }

            let Task { envs, cwd, .. } = task;

            parsed_tasks.insert(
                task_name.clone(),
                Job {
                    task_name,
                    script,
                    envs: global_env.clone().into_iter().chain(envs).collect(),
                    cwd,
                    depends,
                    nexts: Vec::new(),
                },
            );
        }

        Ok(JobSet {
            jobs: parsed_tasks.into_values().collect(),
        })
    }
}

/// Task configuration
pub struct Task {
    /// Environment variables that are specific to this task
    pub envs: HashMap<String, String>,
    /// Script to be executed
    pub script: String,
    /// Working directory
    pub cwd: PathBuf,
    /// Dependencies
    pub depends: Vec<String>,
}

/// Configuration parsing error
#[derive(Debug, thiserror::Error)]
pub enum ConfigParseError {
    /// Task is not executable because of dependency-related issues
    #[error("Task {task_name:?} is not executable because of dependency-related issues")]
    UnexecutableTask { task_name: String },
    /// Task is defined multiple times
    #[error("Task {task_name:?} is defined multiple times")]
    DuplicateTaskName { task_name: String },
    /// Task script parse error
    #[error("Task {task_name:?} script parse error: {error:?}")]
    ScriptParseError {
        task_name: String,
        error: anyhow::Error,
    },
}

pub type TaskResult = Result<(), TaskError>;

#[derive(Clone, Debug, thiserror::Error)]
#[error("Task {task_name:?} failed with exit code {exit_code}")]
pub struct TaskError {
    pub task_name: String,
    pub exit_code: i32,
}

struct JobSet {
    jobs: Vec<Job>,
}

struct Job {
    task_name: String,
    envs: HashMap<String, String>,
    script: SequentialList,
    cwd: PathBuf,

    depends: Vec<Receiver<Result<(), ()>>>,
    nexts: Vec<Sender<Result<(), ()>>>,
}

impl Job {
    async fn execute(self, io: IOSet) -> TaskResult {
        let Job {
            task_name,
            envs,
            script,
            cwd,
            depends,
            nexts,
        } = self;
        if try_join_all(
            depends
                .into_iter()
                .map(|mut rx| async move { rx.recv().await.unwrap() }),
        )
        .await
        .is_err()
        {
            return Ok(());
        }

        let exit_code = deno_task_shell::execute_with_pipes(
            script,
            ShellState::new(envs, &cwd, Default::default(), Default::default()),
            io.stdin,
            io.stdout,
            io.stderr,
        )
        .await;
        if exit_code == 0 {
            for tx in nexts {
                tx.send(Ok(())).await.unwrap();
            }
            Ok(())
        } else {
            Err(TaskError {
                task_name,
                exit_code,
            })
        }
    }
}
