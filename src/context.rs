use std::{fmt::Debug, sync::Arc};

use tokio::sync::{oneshot, Mutex, Notify, RwLock};

use crate::{database::Database, LldResult};

#[derive(Debug, Clone)]
pub enum LldContext {
    Naive(ContextNaive),
    Batching(ContextBatching),
}

impl LldContext {
    pub async fn run(&self) -> LldResult<()> {
        match self {
            LldContext::Naive(context) => context.run().await,
            LldContext::Batching(context) => context.run().await,
        }
    }

    pub async fn request_leasing(
        &self,
        instance_id: String,
        application_id: String,
        duration: u64,
    ) -> LldResult<LeasingResponse> {
        match self {
            LldContext::Naive(context) => {
                context
                    .request_leasing(instance_id, application_id, duration)
                    .await
            }
            LldContext::Batching(context) => {
                context
                    .request_leasing(instance_id, application_id, duration)
                    .await
            }
        }
    }
}

#[derive(Debug)]
pub enum LeasingResponse {
    Granted { validity: u64 },
    Rejected,
}

#[derive(Debug)]
struct ContextQueueEntry {
    instance_id: String,
    application_id: String,
    duration: u64,
    tx: oneshot::Sender<LeasingResponse>,
}

#[derive(Debug, Clone)]
pub struct ContextNaive {
    lock: Arc<Mutex<()>>,
}

impl ContextNaive {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn run(&self) -> LldResult<()> {
        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    pub async fn request_leasing(
        &self,
        instance_id: String,
        application_id: String,
        duration: u64,
    ) -> LldResult<LeasingResponse> {
        debug!(
            "Request leasing for {} with duration {}",
            application_id, duration
        );

        let lease = {
            let _lock = self.lock.lock().await;
            let db = Database::open()?;
            db.request_leasing(&instance_id, &application_id, duration)?
        };

        let response = match lease {
            Some(validity) => LeasingResponse::Granted { validity },
            None => LeasingResponse::Rejected,
        };
        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub struct ContextBatching {
    queue: Arc<RwLock<Vec<ContextQueueEntry>>>,
    notify: Arc<Notify>,
}

impl ContextBatching {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            notify: Arc::new(Notify::new()),
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

    async fn run_task(&self, task: ContextQueueEntry, db: &Database) -> LldResult<()> {
        let lease = db.request_leasing(&task.instance_id, &task.application_id, task.duration)?;

        let response = match lease {
            Some(validity) => LeasingResponse::Granted { validity },
            None => LeasingResponse::Rejected,
        };

        if let Err(e) = task.tx.send(response) {
            error!("Cannot send leasing result to client! ({:?})", e)
        }

        Ok(())
    }

    pub async fn run(&self) -> LldResult<()> {
        loop {
            match self.check_tasks().await {
                Some(tasks) => {
                    let db = Database::open()?;
                    for task in tasks {
                        let x = &db;
                        self.run_task(task, x).await?;
                    }
                }
                None => self.notify.notified().await,
            };
        }
    }

    pub async fn request_leasing(
        &self,
        instance_id: String,
        application_id: String,
        duration: u64,
    ) -> LldResult<LeasingResponse> {
        debug!(
            "Request leasing for {} with duration {}",
            application_id, duration
        );

        let (tx, rx) = oneshot::channel();

        {
            let mut queue = self.queue.write().await;
            queue.push(ContextQueueEntry {
                instance_id,
                application_id,
                duration,
                tx,
            });
        }
        self.notify.notify_one();

        let response = rx.await?;
        Ok(response)
    }
}
