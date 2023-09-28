use std::{cell::OnceCell, collections::HashSet, rc::Rc};

use deno::re_exports::deno_core::{anyhow::Result, futures::future::try_join_all, url::Url};
use log::info;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::config::{Task, TaskName};

#[derive(Clone)]
/// A Task to have in a Job, caching the results of depends and preventing deep copying of Tasks with Rc.
pub struct TaskBuf {
    task: Rc<Task>,
    depends: OnceCell<HashSet<TaskName>>,
    name: TaskName,
}

impl Serialize for TaskBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.task.serialize(serializer)
    }
}

impl TaskBuf {
    /// List of Task names that directly depend on that Task(-Buf)
    pub fn get_depends(&self) -> &HashSet<TaskName> {
        self.depends.get_or_init(|| {
            self.task
                .iter()
                .flat_map(|(atom, _)| atom.config.depends.clone())
                .collect()
        })
    }
    pub fn new(task: Task, name: impl Into<TaskName>) -> Self {
        Self {
            task: Rc::new(task),
            depends: OnceCell::new(),
            name: name.into(),
        }
    }
}

/// A Job that executes a single Task, with dependencies set before execution.
pub struct Job {
    ending_notifier: Sender<(TaskName, Result<()>)>,
    taskbuf: TaskBuf,
}

impl Job {
    pub fn new(taskbuf: TaskBuf, ending_notifier: Sender<(TaskName, Result<()>)>) -> Self {
        Job {
            ending_notifier,
            taskbuf,
        }
    }

    /// Launch the Task. Wait for dependent Tasks. cf: `get_sender`
    pub async fn call(self) {
        info!("{:?} started.", self.taskbuf.name);
        let res = try_join_all(self.taskbuf.task.iter().map(|(atom, path)| {
            let mut url = Url::from_file_path(path.as_ref()).unwrap();
            url.set_fragment(Some(self.taskbuf.name.as_ref()));
            atom.execute(url)
        }))
        .await
        .and(Ok(()));
        info!("{:?} finished.", self.taskbuf.name);
        self.ending_notifier
            .send((self.taskbuf.name, res))
            .await
            .unwrap();
    }
}
