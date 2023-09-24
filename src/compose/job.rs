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
        async move {
            try_join_all(self.task.task.iter().map(|(task, path)| task.execute(path)))
                .await
                .and(Ok(()))
        }
        .await?;
        Ok(())
    }
}
