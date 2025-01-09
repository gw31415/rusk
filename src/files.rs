use std::{
    collections::HashMap,
    fs::File,
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use futures::future::join_all;
use ignore::WalkBuilder;

use crate::rusk::{Rusk, Task};

pub struct ConfigFiles {
    envs: HashMap<String, String>,
    map: HashMap<PathBuf, ConfigFile>,
}

const RUSKFILE: &str = "rusk.toml";

impl ConfigFiles {
    pub fn new(envs: HashMap<String, String>) -> Self {
        Self {
            envs,
            map: Default::default(),
        }
    }
    pub fn tasks_list(&self) -> Vec<(String, PathBuf)> {
        self.map
            .iter()
            .flat_map(|(path, config)| {
                config
                    .tasks
                    .keys()
                    .map(move |name| (name.clone(), path.clone()))
            })
            .collect()
    }
    pub async fn collect(&mut self, path: PathBuf) {
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
                                if ft.is_file() && entry.file_name() == RUSKFILE {
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

impl From<ConfigFiles> for Rusk {
    fn from(config_files: ConfigFiles) -> Self {
        let ConfigFiles { envs, map } = config_files;
        let mut tasks = HashMap::new();
        for (path, config) in map {
            let cwd = path.parent().unwrap();
            for (name, task) in config.tasks {
                tasks.insert(
                    name,
                    Task {
                        envs: task.envs,
                        script: task.script,
                        cwd: task.cwd.unwrap_or_else(|| cwd.to_path_buf()),
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
    pub cwd: Option<PathBuf>,
}
