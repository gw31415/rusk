use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use deno_runtime::deno_core::{
    anyhow::anyhow,
    error::AnyError,
    futures::future::{join_all, try_join_all},
};
use ignore::WalkBuilder;

use crate::config::{RuskFileContent, Task};

use self::job::Job;

pub struct Composer {
    tasks: HashMap<String, Vec<(Task, Arc<PathBuf>)>>,
}

pub const RUSK_FILE: &str = "rusk.toml";

mod job {
    use std::{future::Future, pin::Pin};

    use deno_runtime::deno_core::{
        error::AnyError,
        futures::{
            channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
            StreamExt,
        },
    };
    pub struct Job {
        my_sender: UnboundedSender<()>,
        dependedby: Vec<UnboundedSender<()>>,
        receiver: UnboundedReceiver<()>,
    }

    impl Job {
        pub fn get_sender(&self) -> UnboundedSender<()> {
            self.my_sender.clone()
        }
        pub fn dependedby(&mut self, dependents: UnboundedSender<()>) {
            self.dependedby.push(dependents);
        }
        pub async fn call(
            self,
            boxfuture: Pin<Box<impl Future<Output = Result<(), AnyError>>>>,
        ) -> Result<(), AnyError> {
            drop(self.my_sender);
            let _ = self.receiver.collect::<Vec<_>>().await;
            boxfuture.await?;
            Ok(())
        }
        pub fn new() -> Self {
            let (my_sender, receiver): (UnboundedSender<()>, UnboundedReceiver<()>) =
                mpsc::unbounded::<()>();
            Job {
                my_sender,
                dependedby: Vec::new(),
                receiver,
            }
        }
    }
}

impl Composer {
    /// Perform a Task
    pub async fn execute(&self, name: &str) -> Result<(), AnyError> {
        let mut jobs = self.collect_jobs(name)?;
        let futs = jobs
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .into_iter()
            .map(|name| {
                (
                    name.clone(),
                    Box::pin(async move {
                        try_join_all(
                            unsafe { self.tasks.get(&name).unwrap_unchecked() }
                                .iter()
                                .map(|(task, path)| task.execute(path)),
                        )
                        .await
                        .and(Ok(()))
                    }),
                )
            });
        try_join_all(futs.map(|(name, boxfuture)| {
            unsafe { jobs.remove(&name).unwrap_unchecked() }.call(boxfuture)
        }))
        .await?;
        Ok(())
    }

    /// Obtain dependency structure
    pub fn get_deptree(&self, name: &str) -> Result<HashMap<String, HashSet<String>>, AnyError> {
        let Some(tasks) = self.tasks.get(name) else {
            return Err(anyhow!("Task named {:?} not found.", name));
        };
        let primary_depends: HashSet<_> = tasks
            .iter()
            .flat_map(|(task, _)| task.config.depends.clone())
            .collect();
        let mut depends = primary_depends
            .iter()
            .try_fold(HashMap::new(), |mut parent, n| {
                let subtree = self.get_deptree(n)?;
                for tree in subtree.values() {
                    if tree.contains(name) {
                        return Err(anyhow!("Dependencies around {:?} are inappropriate.", name));
                    }
                }
                parent.extend(subtree);
                Ok(parent)
            })?;
        depends.insert(name.to_owned(), primary_depends);
        Ok(depends)
    }

    fn collect_jobs(&self, name: &str) -> Result<HashMap<String, Job>, AnyError> {
        let tree = self.get_deptree(name)?;
        let names = tree.keys().cloned();
        let mut res: HashMap<_, _> = names.clone().map(|name| (name, Job::new())).collect();
        for name in names {
            let depends = unsafe { tree.get(&name).unwrap_unchecked() };
            for dep in depends {
                let sender = unsafe { res.get(&name).unwrap_unchecked() }.get_sender();
                unsafe { res.get_mut(dep).unwrap_unchecked() }.dependedby(sender);
            }
        }
        Ok(res)
    }

    /// Get all task names.
    pub fn task_names(&self) -> Vec<&String> {
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
                                if ft.is_file() && entry.file_name() == RUSK_FILE {
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
