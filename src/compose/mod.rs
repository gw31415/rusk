use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

use deno::re_exports::deno_runtime::deno_core::{
    anyhow::{anyhow, Result},
    futures::future::join_all,
};
use ignore::WalkBuilder;
use serde::Serialize;
use tokio::{
    sync::mpsc::channel,
    task::{spawn_local, LocalSet},
};

use crate::config::{RuskFileContent, Task, TaskName};

use self::job::{Job, TaskBuf};

/// A structure that searches multiple RUSKFILEs, resolves dependencies, and executes
#[derive(Serialize)]
pub struct Composer {
    tasks: HashMap<TaskName, TaskBuf>,
}

/// RUSKFILE filename
pub const RUSKFILE: &str = "rusk.toml";

mod job;

impl Composer {
    /// Perform a Task
    pub async fn execute(
        &self,
        names: impl IntoIterator<Item = impl Into<TaskName>>,
    ) -> Result<()> {
        let mut deptree: HashMap<_, _> = self
            .get_deptree(names)?
            .into_iter()
            .map(|(k, v)| (k, v.clone()))
            .collect();
        let (sender, mut receiver) = channel(deptree.len());
        let mut jobs: HashMap<_, _> = deptree
            .keys()
            .map(|name| {
                (
                    name.clone(),
                    Job::new(
                        unsafe { self.tasks.get(name).unwrap_unchecked() }.to_owned(),
                        sender.clone(),
                    ),
                )
            })
            .collect();
        drop(sender);
        let local = LocalSet::new();
        local
            .run_until(async move {
                // First time launching Jobs
                for (name, _) in deptree.iter().filter(|(_, deps)| deps.is_empty()) {
                    let job = unsafe { jobs.remove(name).unwrap_unchecked() };
                    spawn_local(job.call());
                }
                while let Some((name, res)) = receiver.recv().await {
                    res?;
                    deptree.remove(&name);
                    for (_, deps) in deptree.iter_mut() {
                        deps.remove(&name);
                    }

                    // Launching a Job with Resolved Dependencies
                    for (name, _) in deptree.iter().filter(|(_, v)| v.is_empty()) {
                        if let Some(job) = jobs.remove(name) {
                            spawn_local(job.call());
                        }
                    }
                }
                Ok(())
            })
            .await
    }

    /// Obtain dependency structure
    // TODO: Circulation detection and error
    pub fn get_deptree(
        &self,
        names: impl IntoIterator<Item = impl Into<TaskName>>,
    ) -> Result<HashMap<TaskName, &HashSet<TaskName>>> {
        let mut depends = HashMap::new();
        for name in names {
            let name = name.into();
            if depends.contains_key(&name) {
                continue;
            }
            let Some(task) = self.tasks.get(&name) else {
                return Err(anyhow!("Task named {:?} not found.", name));
            };
            let primary_depends = task.get_depends();
            depends.extend(self.get_deptree(primary_depends.clone())?);
            depends.insert(name, primary_depends);
        }
        Ok(depends)
    }

    /// Get all task names.
    pub fn task_names(&self) -> Vec<&TaskName> {
        self.tasks.keys().collect()
    }
    /// Initialize Composer with given path.
    pub async fn new(path: impl AsRef<Path>) -> Self {
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
                                            (|| -> Result<_> {
                                                // Read file & deserialize into Config
                                                let content_str =
                                                    io::read_to_string(File::open(&path)?)?;
                                                let content: RuskFileContent =
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
        let mut tasks: HashMap<TaskName, Task> = Default::default();
        for (path, config) in join_all(configfiles).await.into_iter().flatten() {
            let path = Rc::new(path);
            for (name, task) in config.tasks {
                tasks.entry(name).or_default().push((task, path.clone()));
            }
        }
        let tasks = tasks
            .into_iter()
            .map(|(name, task)| (name.clone(), TaskBuf::new(task, name)))
            .collect();
        Composer { tasks }
    }
}
