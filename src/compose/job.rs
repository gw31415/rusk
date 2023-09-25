use std::{cell::OnceCell, collections::HashSet, rc::Rc};

use deno_runtime::deno_core::{
    error::AnyError,
    futures::{
        channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
        future::try_join_all,
        StreamExt,
    },
    url::Url,
};
use log::info;

use crate::config::{Task, TaskName};

/// A Job that executes a single Task, with dependencies set before execution.
pub struct Job {
    my_sender: UnboundedSender<()>,
    receiver: UnboundedReceiver<()>,
    next_jobs: Vec<UnboundedSender<()>>,
    taskbuf: TaskBuf,
}

#[derive(Clone)]
/// A Task to have in a Job, caching the results of depends and preventing deep copying of Tasks with Rc.
pub struct TaskBuf {
    task: Rc<Task>,
    depends: OnceCell<HashSet<TaskName>>,
    name: TaskName,
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

impl From<TaskBuf> for Job {
    fn from(taskbuf: TaskBuf) -> Self {
        let (my_sender, receiver) = mpsc::unbounded::<()>();
        Job {
            my_sender,
            next_jobs: Vec::new(),
            receiver,
            taskbuf,
        }
    }
}

impl Job {
    /// Outputs an UnboundedSender for assignment to the method `dependedby` the dependent Job instance.
    pub fn get_sender(&self) -> UnboundedSender<()> {
        self.my_sender.clone()
    }
    /// Specify the UnboundedSender of the Job that depends on this Job.
    pub fn dependedby(&mut self, dependents: UnboundedSender<()>) {
        self.next_jobs.push(dependents);
    }
    /// Launch the Task. Wait for dependent Tasks. cf: `get_sender`
    pub async fn call(self) -> Result<(), AnyError> {
        drop(self.my_sender);
        let _ = self.receiver.collect::<Vec<_>>().await;
        info!("{:?} started.", self.taskbuf.name);
        try_join_all(self.taskbuf.task.iter().map(|(atom, path)| {
            let mut url = Url::from_file_path(path.as_ref()).unwrap();
            url.set_fragment(Some(self.taskbuf.name.as_ref()));
            atom.execute(url)
        }))
        .await?;
        info!("{:?} finished.", self.taskbuf.name);
        Ok(())
    }
}
