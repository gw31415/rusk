use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use deno_runtime::deno_core::{
    anyhow::anyhow,
    error::AnyError,
    futures::future::{join_all, try_join_all},
};
use ignore::WalkBuilder;

use crate::config::{RuskFileContent, Task};

pub struct Composer {
    tasks: HashMap<String, Vec<(Task, Arc<PathBuf>)>>,
}

const CONFIG_NAME: &str = "rusk.toml";

impl Composer {
    /// Perform a Task
    pub async fn execute(&self, name: &str) -> Result<(), AnyError> {
        let Some(tasks) = self.tasks.get(name) else {
            return Err(anyhow!("Task named {:?} not found.", name));
        };
        try_join_all(tasks.iter().map(|(task, path)| task.execute(path))).await?;
        Ok(())
    }

    /// Get all task names.
    pub fn task_names(&self) -> Vec<&String> {
        self.tasks.keys().collect()
    }
    /// Initialize Composer with given path.
    pub async fn new(path: &str) -> Self {
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
                                if ft.is_file() && entry.file_name() == CONFIG_NAME {
                                    configfiles.lock().unwrap().push({
                                        let path = entry.path().to_path_buf();
                                        // make Future of Config
                                        async {
                                            (|| -> Result<_, AnyError> {
                                                // Read file & deserialize into Config
                                                let mut config = Default::default();
                                                File::open(&path)?.read_to_string(&mut config)?;
                                                let content: RuskFileContent =
                                                    toml::from_str(&config)?;
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
        let mut tasks: HashMap<String, Vec<(Task, Arc<PathBuf>)>> = Default::default();
        for (path, config) in join_all(configfiles).await.into_iter().flatten() {
            let path = Arc::new(path);
            for (name, task) in config.tasks {
                tasks.entry(name).or_default().push((task, path.clone()));
            }
        }
        Composer { tasks }
    }
}
