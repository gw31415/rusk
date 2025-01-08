use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use deno_task_shell::{parser::SequentialList, ShellPipeReader, ShellPipeWriter, ShellState};
use futures::future::try_join_all;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, thiserror::Error)]
pub enum RuskError {
    #[error(transparent)]
    TaskError(#[from] TaskError),
    #[error(transparent)]
    ConfigParseError(#[from] ConfigParseError),
}

pub struct Config<'a> {
    pub tasks: HashMap<String, Task<'a>>,
}

impl Config<'_> {
    pub async fn execute(
        self,
        stdin: ShellPipeReader,
        stdout: ShellPipeWriter,
        stderr: ShellPipeWriter,
    ) -> Result<(), RuskError> {
        let parsed = ParsedConfig::try_from(self)?;
        let fs = parsed.tasks.into_iter().map(|task| {
            let (stdin, stdout, stderr) = (stdin.clone(), stdout.clone(), stderr.clone());
            task.execute(stdin, stdout, stderr)
        });
        try_join_all(fs).await?;
        Ok(())
    }
}

pub struct Task<'a> {
    pub envs: HashMap<String, String>,
    pub script: Cow<'a, str>,
    pub cwd: PathBuf,
    pub depends: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigParseError {
    #[error("Task {task_name} is not executable because of dependency-related issues")]
    UnexecutableTask { task_name: String },
    #[error("Task {task_name} is defined multiple times")]
    DuplicateTaskName { task_name: String },
    #[error("Task {task_name} script parse error: {error}")]
    ScriptParseError {
        task_name: String,
        error: anyhow::Error,
    },
}

impl TryFrom<Config<'_>> for ParsedConfig {
    type Error = ConfigParseError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let Config { tasks } = config;
        let mut tasks = tasks.into_iter().collect::<Vec<_>>();
        tasks.sort_by(|(_, t1), (_, t2)| t1.depends.len().cmp(&t2.depends.len()));

        let mut parsed_tasks: HashMap<String, ParsedTask> = HashMap::new();

        for (task_name, task) in tasks {
            if parsed_tasks.contains_key(&task_name) {
                return Err(ConfigParseError::DuplicateTaskName { task_name });
            }

            let script = match deno_task_shell::parser::parse(&task.script) {
                Ok(script) => script,
                Err(error) => return Err(ConfigParseError::ScriptParseError { task_name, error }),
            };

            let mut depends = Vec::new();
            for dep_name in &task.depends {
                let (tx, rx) = channel::<Result<(), ()>>(1);
                depends.push(rx);
                if let Some(dep_task) = parsed_tasks.get_mut(dep_name) {
                    dep_task.nexts.push(tx);
                } else {
                    return Err(ConfigParseError::UnexecutableTask { task_name });
                }
            }

            let Task { envs, cwd, .. } = task;

            parsed_tasks.insert(
                task_name.clone(),
                ParsedTask {
                    task_name,
                    script,
                    envs,
                    cwd,
                    depends,
                    nexts: Vec::new(),
                },
            );
        }

        Ok(ParsedConfig {
            tasks: parsed_tasks.into_values().collect(),
        })
    }
}

pub type TaskResult = Result<(), TaskError>;

#[derive(Clone, Debug, thiserror::Error)]
#[error("Task {task_name} failed with exit code {exit_code}")]
pub struct TaskError {
    pub task_name: String,
    pub exit_code: i32,
}

struct ParsedConfig {
    tasks: Vec<ParsedTask>,
}

struct ParsedTask {
    task_name: String,
    envs: HashMap<String, String>,
    script: SequentialList,
    cwd: PathBuf,

    depends: Vec<Receiver<Result<(), ()>>>,
    nexts: Vec<Sender<Result<(), ()>>>,
}

impl ParsedTask {
    async fn execute(
        self,
        stdin: ShellPipeReader,
        stdout: ShellPipeWriter,
        stderr: ShellPipeWriter,
    ) -> TaskResult {
        let ParsedTask {
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
            stdin,
            stdout,
            stderr,
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

#[tokio::main]
async fn main() {
    let envs: HashMap<_, _> = std::env::vars().collect();
    let cwd = std::env::current_dir().unwrap();
    let config = Config {
        tasks: [
            (
                "task1".to_string(),
                Task {
                    envs: envs.clone(),
                    script: "false && echo 'task1 start' && sleep 2 && echo 'task1 done'".into(),
                    cwd: cwd.clone(),
                    depends: vec![],
                },
            ),
            (
                "task2".to_string(),
                Task {
                    envs: envs.clone(),
                    script: "echo 'task2 start' && sleep 1 && echo 'task2 done'".into(),
                    cwd: cwd.clone(),
                    depends: vec![], // vec!["task1".to_string()],
                },
            ),
        ]
        .into(),
    };
    match config
        .execute(
            ShellPipeReader::stdin(),
            ShellPipeWriter::stdout(),
            ShellPipeWriter::stderr(),
        )
        .await
    {
        Ok(()) => println!("All tasks are done"),
        Err(e) => match e {
            RuskError::TaskError(e) => {
                eprintln!("{e}");
                std::process::exit(e.exit_code);
            }
            RuskError::ConfigParseError(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
    }
}
