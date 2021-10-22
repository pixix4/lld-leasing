use std::sync::Arc;

use tokio::sync::{oneshot, Notify, RwLock};

use crate::{database::Database, LldResult};

#[derive(Debug)]
pub enum LeasingResponse {
    Success {
        instance_id: String,
        application_id: String,
        validity: i64,
    },
    Error {
        instance_id: String,
        application_id: String,
    },
}

#[derive(Debug)]
struct ContextQueueEntry {
    instance_id: String,
    application_id: String,
    tx: oneshot::Sender<LeasingResponse>,
}

#[derive(Debug, Clone)]
pub struct Context {
    queue: Arc<RwLock<Vec<ContextQueueEntry>>>,
    notify: Arc<Notify>,
}

impl Context {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn run(&self, db: Database) -> LldResult<()> {
        loop {
            match self.check_tasks().await {
                Some(tasks) => self.run_tasks(tasks, &db).await?,
                None => self.notify.notified().await,
            };
        }
    }

    async fn check_tasks(&self) -> Option<Vec<ContextQueueEntry>> {
        let mut queue = self.queue.write().await;
        let len = queue.len();
        if len > 0 {
            Some(queue.drain(0..len).collect())
        } else {
            None
        }
    }

    async fn run_tasks(&self, tasks: Vec<ContextQueueEntry>, db: &Database) -> LldResult<()> {
        for task in tasks {
            let lease = db.request_leasing(&task.instance_id, &task.application_id)?;

            let response = match lease {
                Some(validity) => LeasingResponse::Success {
                    instance_id: task.instance_id,
                    application_id: task.application_id,
                    validity,
                },
                None => LeasingResponse::Error {
                    instance_id: task.instance_id,
                    application_id: task.application_id,
                },
            };

            if let Err(e) = task.tx.send(response) {
                eprintln!("Cannot send leasing result to client! ({:?})", e)
            }
        }

        Ok(())
    }

    pub async fn request_leasing(
        &self,
        instance_id: String,
        application_id: String,
    ) -> oneshot::Receiver<LeasingResponse> {
        let (tx, rx) = oneshot::channel();

        let mut queue = self.queue.write().await;
        queue.push(ContextQueueEntry {
            instance_id,
            application_id,
            tx,
        });
        self.notify.notify_one();

        rx
    }
}
