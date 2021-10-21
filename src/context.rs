use std::sync::Arc;

use tokio::sync::{oneshot, Notify, RwLock};

use crate::{database::Database, LldResult};

#[derive(Debug)]
pub enum LeasingResponse {
    Success { id: String, validity: i64 },
    Error { id: String },
}

#[derive(Debug)]
struct ContextQueueEntry {
    id: String,
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
        println!("---- run tasks ----");
        for task in &tasks {
            println!("Request for {}", task.id);
        }

        for task in tasks {
            let lease = db.request_leasing(&task.id)?;

            let response = match lease {
                Some(validity) => LeasingResponse::Success {
                    id: task.id,
                    validity,
                },
                None => LeasingResponse::Error { id: task.id },
            };

            if let Err(e) = task.tx.send(response) {
                eprintln!("Cannot send leasing result to client! ({:?})", e)
            }
        }

        Ok(())
    }

    pub async fn request_leasing(&self, id: String) -> oneshot::Receiver<LeasingResponse> {
        let (tx, rx) = oneshot::channel();

        let mut queue = self.queue.write().await;
        queue.push(ContextQueueEntry { id, tx });
        self.notify.notify_one();

        rx
    }
}
