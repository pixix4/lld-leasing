use std::{fmt::Debug, sync::Arc};

use tokio::sync::{oneshot, Notify, RwLock};

use crate::{
    cache::{CacheResult, ContextCache},
    context::LeasingResponse,
    database::{Database, DatabaseTask},
    LldResult,
};

#[derive(Debug)]
pub struct QueueEntry {
    task: DatabaseTask,
    tx: oneshot::Sender<bool>,
}

#[derive(Debug, Clone)]
pub struct ContextBatching {
    queue: Arc<RwLock<Vec<QueueEntry>>>,
    notify: Arc<Notify>,
    cache: ContextCache,
}

impl ContextBatching {
    #[allow(clippy::new_without_default)]
    pub fn new(db: Database) -> LldResult<Self> {
        let cache = ContextCache::new(&db)?;
        Ok(Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            notify: Arc::new(Notify::new()),
            cache,
        })
    }

    async fn check_tasks(&self) -> Option<Vec<QueueEntry>> {
        let mut queue = self.queue.write().await;
        let len = queue.len();
        if len > 0 {
            Some(queue.drain(0..len).collect())
        } else {
            None
        }
    }

    pub async fn run(&self) -> LldResult<()> {
        loop {
            match self.check_tasks().await {
                Some(entries) => {
                    let db = Database::open()?;

                    let mut tasks = Vec::<DatabaseTask>::with_capacity(entries.len());
                    let mut callbacks = Vec::<oneshot::Sender<bool>>::with_capacity(entries.len());

                    for entry in entries {
                        tasks.push(entry.task);
                        callbacks.push(entry.tx);
                    }

                    let result = db.execute_tasks(&tasks)?;

                    for tx in callbacks {
                        if let Err(e) = tx.send(result) {
                            error!("Cannot send leasing result to client! ({:?})", e)
                        }
                    }
                }
                None => self.notify.notified().await,
            };
        }
    }

    pub async fn request_leasing(
        &self,
        application_id: String,
        instance_id: String,
        duration: u64,
        now: u64,
    ) -> LldResult<LeasingResponse> {
        let cache_result = self
            .cache
            .request_leasing(application_id, instance_id, duration, now)
            .await?;

        let (task, validity) = match cache_result {
            CacheResult::Rejected => return Ok(LeasingResponse::Rejected),
            CacheResult::GrantedInsert {
                application_id,
                instance_id,
                validity,
            } => (
                DatabaseTask::Insert {
                    application_id,
                    instance_id,
                    validity,
                },
                validity,
            ),
            CacheResult::GrantedUpdate {
                application_id,
                instance_id,
                validity,
            } => (
                DatabaseTask::Update {
                    application_id,
                    instance_id,
                    validity,
                },
                validity,
            ),
        };

        let (tx, rx) = oneshot::channel();
        {
            let mut queue = self.queue.write().await;
            queue.push(QueueEntry { task, tx });
        }
        self.notify.notify_one();

        rx.await?;
        Ok(LeasingResponse::Granted { validity })
    }
}
