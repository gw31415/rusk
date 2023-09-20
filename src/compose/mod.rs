use std::{
    collections::{HashMap, HashSet},
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

use self::job::Job;

pub struct Composer {
    tasks: HashMap<String, Vec<(Task, Arc<PathBuf>)>>,
}

const CONFIG_NAME: &str = "rusk.toml";

mod job {
    use std::{future::Future, pin::Pin};

    use deno_runtime::deno_core::{
        error::AnyError,
        futures::channel::mpsc::{self, Receiver, Sender},
    };
    pub struct Job {
        my_sender: Sender<()>,
        interdependents: Vec<Sender<()>>,
        receiver: Receiver<()>,
    }

    impl Job {
        pub fn get_sender(&self) -> Sender<()> {
            self.my_sender.clone()
        }
        pub fn set_interdependents(&mut self, dependents: impl IntoIterator<Item = Sender<()>>) {
            self.interdependents = dependents.into_iter().collect();
        }
        pub async fn call(
            mut self,
            job: Pin<Box<impl Future<Output = Result<(), AnyError>>>>,
        ) -> Result<(), AnyError> {
            self.my_sender.close_channel();
            while let Ok(Some(_)) = self.receiver.try_next() {}
            job.await?;
            for mut sender in self.interdependents {
                sender.close_channel();
            }
            Ok(())
        }
        pub fn new() -> Self {
            let (my_sender, receiver): (Sender<()>, Receiver<()>) = mpsc::channel(0);
            Job {
                my_sender,
                interdependents: Vec::new(),
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
                            self.tasks
                                .get(&name)
                                .unwrap()
                                .iter()
                                .map(|(task, path)| task.execute(path)),
                        )
                        .await
                        .and(Ok(()))
                    }),
                )
            });
        try_join_all(futs.map(|(name, boxfuture)| jobs.remove(&name).unwrap().call(boxfuture)))
            .await?;
        Ok(())
    }

    fn collect_jobs(&self, name: &str) -> Result<HashMap<String, Job>, AnyError> {
        fn get_deptree(
            composer: &Composer,
            name: &str,
        ) -> Result<HashMap<String, HashSet<String>>, AnyError> {
            let Some(tasks) = composer.tasks.get(name) else {
                return Err(anyhow!("Task named {:?} not found.", name));
            };
            let primary_depends: HashSet<_> = tasks
                .iter()
                .flat_map(|(task, _)| task.config.depends.clone())
                .collect();
            let mut depends =
                primary_depends
                    .iter()
                    .try_fold(HashMap::new(), |mut parent, n| {
                        let a = get_deptree(composer, n)?;
                        if a.contains_key(name) {
                            return Err(anyhow!(
                                "Dependencies around {:?} are inappropriate.",
                                name
                            ));
                        }
                        parent.extend(a);
                        Ok(parent)
                    })?;
            depends.insert(name.to_owned(), primary_depends);
            Ok(depends)
        }
        let tree = get_deptree(self, name)?;
        let names = tree.keys().cloned();
        let mut res: HashMap<_, _> = names.clone().map(|name| (name, Job::new())).collect();
        let mut interdependents: HashMap<_, _> = names
            .into_iter()
            .map(|name| {
                let senders: Vec<_> = tree
                    .get(&name)
                    .unwrap()
                    .iter()
                    .map(|name| res.get(name).unwrap().get_sender())
                    .to_owned()
                    .collect();
                (name, senders)
            })
            .collect();
        for (name, job) in res.iter_mut() {
            job.set_interdependents(interdependents.remove(name).unwrap());
        }
        Ok(res)
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
