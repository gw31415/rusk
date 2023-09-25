use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

use deno_runtime::deno_core::{
    anyhow::anyhow,
    error::AnyError,
    futures::future::{join_all, try_join_all},
};
use ignore::WalkBuilder;

use crate::config::{RuskFileContent, Task, TaskName};

use self::job::{Job, TaskBuf};

pub struct Composer {
    tasks: HashMap<TaskName, TaskBuf>,
}

pub const RUSKFILE: &str = "rusk.toml";

mod job;

impl Composer {
    /// Perform a Task
    pub async fn execute(&self, name: impl Into<TaskName>) -> Result<(), AnyError> {
        try_join_all(self.collect_jobs(name.into())?.map(|job| job.call())).await?;
        Ok(())
    }

    /// Obtain dependency structure
    // TODO: Circulation detection and error
    pub fn get_deptree(
        &self,
        name: impl Into<TaskName>,
    ) -> Result<HashMap<TaskName, &HashSet<TaskName>>, AnyError> {
        let name = name.into();
        let Some(task) = self.tasks.get(&name) else {
            return Err(anyhow!("Task named {:?} not found.", name));
        };
        let primary_depends: &HashSet<_> = task.get_depends();
        let mut depends = primary_depends
            .iter()
            .try_fold(HashMap::new(), |mut parent, n| {
                parent.extend(self.get_deptree(n.clone())?);
                Ok::<_, AnyError>(parent)
            })?;
        depends.insert(name, primary_depends);
        Ok(depends)
    }

    fn collect_jobs(&self, name: TaskName) -> Result<impl Iterator<Item = Job>, AnyError> {
        let mut res = HashMap::new();
        for (name, depends) in self.get_deptree(name)? {
            macro_rules! get_or_insert_mut_job {
                ($name: expr) => {{
                    // Existence of job named `name` is checked in `self.get_deptree`.
                    let job = unsafe { self.tasks.get(&$name).unwrap_unchecked() };
                    res.entry($name).or_insert(job.clone().into())
                }};
            }

            let sender = {
                let master: &mut Job = get_or_insert_mut_job!(name);
                master.get_sender()
            };

            for dep in depends.iter().cloned() {
                get_or_insert_mut_job!(dep).dependedby(sender.clone());
            }
        }
        Ok(res.into_values())
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
                                            (|| -> Result<_, AnyError> {
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
