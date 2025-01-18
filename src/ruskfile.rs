use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::Error;
use colored::Colorize;
use futures::future::join_all;
use ignore::WalkBuilder;
use toml::Table;

use crate::rusk::Task;

/// Configuration files
#[derive(Default)]
pub struct RuskfileComposer {
    map: HashMap<PathBuf, RuskfileDeserializer>,
}

/// Check if the filename is ruskfile
macro_rules! is_ruskfile {
    ($f: expr) => {
        matches!($f, "rusk.toml" | ".rusk.toml")
    };
}

/// Item of tasks_list
pub struct TasksListItem<'a> {
    /// Task name
    name: &'a str,
    /// Task description
    description: Option<&'a str>,
    /// Path to rusk.toml
    path: &'a Path,
}

impl Display for TasksListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t", self.name)?;
        if let Some(description) = &self.description {
            write!(f, "{} ", description.bold())?;
        }
        write!(
            f,
            "({} {})",
            "in".dimmed().italic(),
            self.path.to_string_lossy().yellow().dimmed().italic(),
        )
    }
}

impl RuskfileComposer {
    /// Create a new Ruskfiles
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    /// List all tasks
    pub fn tasks_list(&self) -> impl Iterator<Item = TasksListItem<'_>> {
        self.map.iter().flat_map(|(path, config)| {
            config.tasks.iter().map(move |(name, task)| TasksListItem {
                name,
                description: task.description.as_deref(),
                path,
            })
        })
    }
    /// Walk through the directory and find all rusk.toml files
    pub async fn walkdir(&mut self, path: PathBuf) {
        let configfiles = {
            let configfiles: Arc<Mutex<Vec<_>>> = Default::default();
            WalkBuilder::new(path)
                .require_git(true)
                .follow_links(true)
                .build_parallel()
                .run(|| {
                    Box::new(|res| {
                        if let Ok(entry) = res {
                            if let Some(ft) = entry.file_type() {
                                if ft.is_file()
                                    && is_ruskfile!(entry.file_name().to_str().unwrap_or(""))
                                {
                                    configfiles.lock().unwrap().push({
                                        let path = entry.path().to_path_buf();
                                        // make Future of Config
                                        async {
                                            (|| -> Result<_, Error> {
                                                // Read file & deserialize into Config
                                                let content_str =
                                                    io::read_to_string(File::open(&path)?)?;
                                                let content: RuskfileDeserializer =
                                                    toml::from_str(&content_str)?;
                                                Ok((path, content))
                                            })()
                                            .ok()
                                        }
                                    });
                                }
                                return ignore::WalkState::Continue;
                            }
                        }
                        ignore::WalkState::Skip
                    })
                });
            Arc::try_unwrap(configfiles)
                .ok()
                .unwrap()
                .into_inner()
                .unwrap()
        };
        let map: HashMap<_, _> = join_all(configfiles).await.into_iter().flatten().collect();
        self.map.extend(map);
    }
}

impl From<RuskfileComposer> for HashMap<String, Task> {
    fn from(composer: RuskfileComposer) -> Self {
        let RuskfileComposer { map } = composer;
        let mut tasks = HashMap::new();
        for (path, config) in map {
            let configfile_dir = path.parent().unwrap();
            for (name, TaskDeserializer { inner, .. }) in config.tasks {
                let TaskDeserializerInner {
                    envs,
                    script,
                    depends,
                    cwd,
                } = inner.try_into().unwrap(); // NOTE: It is guaranteed to be a table, and fields that are not present will have default values.
                tasks.insert(
                    name,
                    Task {
                        envs,
                        script,
                        cwd: if let Some(cwd) = cwd {
                            configfile_dir.join(cwd)
                        } else {
                            configfile_dir.to_path_buf()
                        },
                        depends,
                    },
                );
            }
        }
        tasks
    }
}

#[derive(serde::Deserialize)]
struct RuskfileDeserializer {
    #[serde(default)]
    tasks: HashMap<String, TaskDeserializer>,
}

#[derive(serde::Deserialize)]
struct TaskDeserializer {
    #[serde(flatten)]
    inner: Table,
    /// Description
    #[serde(default)]
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct TaskDeserializerInner {
    /// Environment variables that are specific to this task
    #[serde(default)]
    envs: HashMap<String, String>,
    /// Script to be executed
    script: Option<String>,
    /// Dependencies
    #[serde(default)]
    depends: Vec<String>,
    /// Working directory
    #[serde(default)]
    cwd: Option<String>,
}
