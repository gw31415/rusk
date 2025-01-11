use std::{
    borrow::Cow,
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

use crate::rusk::{Rusk, Task};

/// Configuration files
pub struct RuskConfigFiles {
    envs: HashMap<String, String>,
    map: HashMap<PathBuf, ConfigFile>,
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
    pub name: Cow<'a, str>,
    /// Task description
    pub description: Option<Cow<'a, str>>,
    /// Path to rusk.toml
    pub path: Cow<'a, Path>,
}

impl Display for TasksListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t", self.name)?;
        if let Some(description) = &self.description {
            write!(f, "{} ", description.bold())?;
        }
        write!(
            f,
            "{} {}",
            "in".dimmed().italic(),
            self.path.to_string_lossy().dimmed().yellow().italic(),
        )
    }
}

impl RuskConfigFiles {
    /// Create a new RuskConfigFiles
    pub fn new(envs: HashMap<String, String>) -> Self {
        Self {
            envs,
            map: Default::default(),
        }
    }
    /// List all tasks
    pub fn tasks_list(&self) -> impl Iterator<Item = TasksListItem<'_>> {
        self.map.iter().flat_map(|(path, config)| {
            config.tasks.iter().map(move |(name, task)| TasksListItem {
                name: Cow::Borrowed(name),
                description: task.description.as_deref().map(Cow::Borrowed),
                path: Cow::Borrowed(path),
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
                                                let content: ConfigFile =
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

impl From<RuskConfigFiles> for Rusk {
    fn from(config_files: RuskConfigFiles) -> Self {
        let RuskConfigFiles { envs, map } = config_files;
        let mut tasks = HashMap::new();
        for (path, config) in map {
            let configfile_dir = path.parent().unwrap();
            for (name, task) in config.tasks {
                tasks.insert(
                    name,
                    Task {
                        envs: task.envs,
                        script: task.script,
                        cwd: if let Some(cwd) = task.cwd {
                            configfile_dir.join(cwd)
                        } else {
                            configfile_dir.to_path_buf()
                        },
                        depends: task.depends,
                    },
                );
            }
        }
        Self { tasks, envs }
    }
}

#[derive(serde::Deserialize)]
struct ConfigFile {
    #[serde(default)]
    pub tasks: HashMap<String, ConfigTask>,
}

#[derive(serde::Deserialize)]
struct ConfigTask {
    /// Environment variables that are specific to this task
    #[serde(default)]
    pub envs: HashMap<String, String>,
    /// Script to be executed
    pub script: String,
    /// Dependencies
    #[serde(default)]
    pub depends: Vec<String>,
    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
}
