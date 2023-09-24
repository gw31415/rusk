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

use crate::config::{RuskFileContent, Task};

use self::job::{Job, TaskBuf};

pub struct Composer {
    tasks: HashMap<String, TaskBuf>,
}

pub const RUSK_FILE: &str = "rusk.toml";

mod job {
    use std::{cell::OnceCell, collections::HashSet, sync::Arc};

    use deno_runtime::deno_core::{
        error::AnyError,
        futures::{
            channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
            future::try_join_all,
            StreamExt,
        },
    };

    use crate::config::Task;

    pub struct Job {
        my_sender: UnboundedSender<()>,
        receiver: UnboundedReceiver<()>,
        next_jobs: Vec<UnboundedSender<()>>,
        task: TaskBuf,
    }

    #[derive(Clone)]
    pub struct TaskBuf {
        task: Arc<Task>,
        depends: OnceCell<HashSet<String>>,
    }

    impl TaskBuf {
        pub fn get_depends(&self) -> &HashSet<String> {
            self.depends.get_or_init(|| {
                self.task
                    .iter()
                    .flat_map(|(task, _)| task.config.depends.clone())
                    .collect()
            })
        }
        pub fn new(task: Task) -> Self {
            Self {
                task: Arc::new(task),
                depends: OnceCell::new(),
            }
        }
    }

    impl From<TaskBuf> for Job {
        fn from(val: TaskBuf) -> Self {
            let (my_sender, receiver) = mpsc::unbounded::<()>();
            Job {
                my_sender,
                next_jobs: Vec::new(),
                receiver,
                task: val,
            }
        }
    }

    impl Job {
        pub fn get_sender(&self) -> UnboundedSender<()> {
            self.my_sender.clone()
        }
        pub fn dependedby(&mut self, dependents: UnboundedSender<()>) {
            self.next_jobs.push(dependents);
        }
        pub async fn call(self) -> Result<(), AnyError> {
            drop(self.my_sender);
            let _ = self.receiver.collect::<Vec<_>>().await;
            async move {
                try_join_all(self.task.task.iter().map(|(task, path)| task.execute(path)))
                    .await
                    .and(Ok(()))
            }
            .await?;
            Ok(())
        }
    }
}

impl Composer {
    /// Perform a Task
    pub async fn execute(&self, name: &str) -> Result<(), AnyError> {
        try_join_all(self.collect_jobs(name)?.map(|job| job.call())).await?;
        Ok(())
    }

    /// Obtain dependency structure
    pub fn get_deptree<'c>(
        &'c self,
        name: &str,
    ) -> Result<HashMap<String, &'c HashSet<String>>, AnyError> {
        let Some(task) = self.tasks.get(name) else {
            return Err(anyhow!("Task named {:?} not found.", name));
        };
        let primary_depends: &HashSet<_> = task.get_depends();
        let mut depends = primary_depends
            .iter()
            .try_fold(HashMap::new(), |mut parent, n| {
                let subtree = self.get_deptree(n)?;
                for children in subtree.values() {
                    if children.contains(name) {
                        return Err(anyhow!("Dependencies around {:?} are inappropriate.", name));
                    }
                }
                parent.extend(subtree);
                Ok(parent)
            })?;
        depends.insert(name.to_owned(), primary_depends);
        Ok(depends)
    }

    fn collect_jobs(&self, name: &str) -> Result<impl Iterator<Item = Job>, AnyError> {
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
        let mut tasks: HashMap<String, Task> = Default::default();
        for (path, config) in join_all(configfiles).await.into_iter().flatten() {
            let path = Rc::new(path);
            for (name, task) in config.tasks {
                tasks.entry(name).or_default().push((task, path.clone()));
            }
        }
        let tasks = tasks
            .into_iter()
            .map(|(name, source)| (name, TaskBuf::new(source)))
            .collect();
        Composer { tasks }
    }
}
