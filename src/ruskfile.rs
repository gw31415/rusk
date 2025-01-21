use std::{
    fmt::Display,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use colored::Colorize;
use futures::future::join_all;
use hashbrown::{hash_map::EntryRef, HashMap};
use ignore::{WalkBuilder, WalkState};
use toml::Table;

use crate::{path::NormarizedPath, rusk::Task};

/// Configuration files
#[derive(Default)]
pub struct RuskfileComposer {
    /// Map of rusk.toml files
    map: HashMap<NormarizedPath, Result<RuskfileDeserializer, String>>,
}

/// Check if the filename is ruskfile
macro_rules! is_ruskfile {
    ($f: expr) => {
        matches!($f, "rusk.toml" | ".rusk.toml")
    };
}

/// Item of tasks_list
#[derive(PartialEq, Eq, PartialOrd)]
pub struct TasksListItem<'a> {
    /// Task content
    content: Result<TaskListItemContent<'a>, &'a str>,
    /// Path to rusk.toml
    path: &'a NormarizedPath,
}

impl TasksListItem<'_> {
    /// Write verbose error
    pub fn verbose(&self) -> impl Display + '_ {
        if self.content.is_ok() {
            panic!("TasksListItem::verbose() is not for Ok variant");
        }
        TaskErrorVerboseDisplay(self)
    }
}

/// Struct which implements Display to show error verbose
struct TaskErrorVerboseDisplay<'a>(&'a TasksListItem<'a>);

impl Display for TaskErrorVerboseDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        match inner.content {
            Err(err) => {
                // Error Title: Decorated path
                inner
                    .path
                    .as_rel_str()
                    .yellow()
                    .bold()
                    .italic()
                    .underline()
                    .fmt(f)?;

                ':'.fmt(f)?;

                // Indented error message
                for line in err.lines() {
                    "\n\t".fmt(f)?;
                    line.fmt(f)?;
                }
            }
            _ => unimplemented!(),
        };
        Ok(())
    }
}

impl Ord for TasksListItem<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let cmp = self.content.cmp(&other.content);
        if let std::cmp::Ordering::Equal = cmp {
            self.path.cmp(other.path)
        } else {
            cmp
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd)]
struct TaskListItemContent<'a> {
    /// Task name
    name: &'a str,
    /// Task description
    description: Option<&'a str>,
}

impl Ord for TaskListItemContent<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(other.name)
    }
}

impl Display for TasksListItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ////////////////////////////////////////////////
        //
        // Format:
        //     (task_name)\t(description)\t"in "(path)
        //
        ////////////////////////////////////////////////

        /// write content with tab
        macro_rules! writet {
            ($x: expr) => {
                $x.fmt(f)?;
                '\t'.fmt(f)?;
            };
        }

        match self.content {
            Ok(TaskListItemContent { name, description }) => {
                // (task_name)
                writet!(name);
                if let Some(description) = description {
                    // (description)
                    writet!(description.bold());
                }
            }
            Err(_) => {
                // (task_name): Undefined Task
                writet!("(null)".dimmed().italic());
            }
        }

        // "in "
        "in".dimmed().italic().fmt(f)?;
        ' '.fmt(f)?;

        // (path)
        self.path.as_rel_str().yellow().dimmed().italic().fmt(f)
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
        self.map
            .iter()
            .filter_map(|(path, res)| match res {
                Ok(config) => Some(config.tasks.iter().map(move |(name, task)| TasksListItem {
                    content: Ok(TaskListItemContent {
                        name,
                        description: task.description.as_deref(),
                    }),
                    path,
                })),
                _ => None,
            })
            .flatten()
    }
    /// List all errors
    pub fn errors_list(&self) -> impl Iterator<Item = TasksListItem<'_>> {
        self.map.iter().filter_map(|(path, res)| match res {
            Err(err) => Some(TasksListItem {
                content: Err(err),
                path,
            }),
            _ => None,
        })
    }

    /// Walk through the directory and find all rusk.toml files
    pub async fn walkdir(&mut self, path: impl AsRef<Path>) {
        let loading_confs = {
            let futures_collect: Arc<Mutex<Vec<_>>> = Default::default();
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
                                    let path = entry.path().to_path_buf();
                                    futures_collect.lock().unwrap().push({
                                        // make Future of Config
                                        async move {
                                            let res = tokio::fs::read_to_string(&path)
                                                .await
                                                .map_err(Error::from)
                                                .and_then(|content| {
                                                    toml::from_str::<RuskfileDeserializer>(&content)
                                                        .map_err(Error::from)
                                                })
                                                .map_err(|err| err.to_string());
                                            (path.into(), res)
                                        }
                                    });
                                }
                                return WalkState::Continue;
                            }
                        }
                        WalkState::Skip
                    })
                });
            Arc::try_unwrap(futures_collect)
                .ok()
                .unwrap()
                .into_inner()
                .unwrap()
        };
        let map: HashMap<NormarizedPath, _> = join_all(loading_confs).await.into_iter().collect();
        self.map.extend(map);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuskfileConvertError {
    #[error("Task {0:?} is duplicated")]
    DuplicatedTaskName(String),
}

impl TryFrom<RuskfileComposer> for HashMap<String, Task> {
    type Error = RuskfileConvertError;
    fn try_from(composer: RuskfileComposer) -> Result<Self, Self::Error> {
        let RuskfileComposer { map } = composer;
        let mut tasks = HashMap::new();
        for (path, res) in map {
            let Ok(config) = res else {
                continue;
            };
            let configfile_dir = path.parent().unwrap(); // NOTE: path is guaranteed to be a NormalizedPath of an existing file, so it should have a parent directory
            for (name, TaskDeserializer { inner, .. }) in config.tasks {
                let TaskDeserializerInner {
                    envs,
                    script,
                    depends,
                    cwd,
                } = inner.try_into().unwrap(); // NOTE: It is guaranteed to be a table, and fields that are not present will have default values.
                match tasks.entry_ref(&name) {
                    EntryRef::Occupied(_) => {
                        return Err(RuskfileConvertError::DuplicatedTaskName(name));
                    }
                    EntryRef::Vacant(e) => {
                        e.insert(Task {
                            envs,
                            script,
                            cwd: configfile_dir.join(cwd).into(),
                            depends,
                        });
                    }
                }
            }
        }
        Ok(tasks)
    }
}

/// serde::Deserialize of Ruskfile File content
#[derive(serde::Deserialize)]
struct RuskfileDeserializer {
    /// TaskDeserializers map
    #[serde(default)]
    tasks: HashMap<String, TaskDeserializer>,
}

/// serde::Deserialize of Each rusk Task
#[derive(serde::Deserialize)]
struct TaskDeserializer {
    /// Task Raw content
    #[serde(flatten)]
    inner: Table,
    /// Description for help
    #[serde(default)]
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct TaskDeserializerInner {
    /// Environment variables that are specific to this task
    #[serde(default)]
    envs: HashMap<String, String>,
    /// Script to be executed
    #[serde(default)]
    script: Option<String>,
    /// Dependencies
    #[serde(default)]
    depends: Vec<String>,
    /// Working directory
    #[serde(default)]
    cwd: String,
}

impl Default for TaskDeserializerInner {
    fn default() -> Self {
        Self {
            envs: Default::default(),
            script: Default::default(),
            depends: Default::default(),
            cwd: ".".to_string(),
        }
    }
}
