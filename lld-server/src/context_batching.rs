use std::{fmt::Debug, sync::Arc};

use tokio::sync::{oneshot, Mutex, Notify, RwLock};

use crate::{
    cache::{CacheResult, ContextCache},
    context::LeasingResponse,
    database::{Database, DatabaseTask},
    LldResult,
};

#[derive(Debug)]
pub struct QueueEntry {
    pub task: DatabaseTask,
    pub tx: oneshot::Sender<LeasingResponse>,
}

#[derive(Debug, Clone)]
pub struct ContextBatching {
    queue: Arc<RwLock<Vec<QueueEntry>>>,
    notify: Arc<Notify>,
    db: Arc<Mutex<Database>>,
    cache: ContextCache,
}

impl ContextBatching {
    #[allow(clippy::new_without_default)]
    pub fn new(db: Database) -> LldResult<Self> {
        let cache = ContextCache::new(&db)?;
        Ok(Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            notify: Arc::new(Notify::new()),
            db: Arc::new(Mutex::new(db)),
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
        let db = self.db.lock().await;
        loop {
            match self.check_tasks().await {
                Some(entries) => {
                    let mut tasks = Vec::<DatabaseTask>::with_capacity(entries.len());
                    let mut callbacks =
                        Vec::<(u64, oneshot::Sender<LeasingResponse>)>::with_capacity(
                            entries.len(),
                        );

                    for entry in entries {
                        let QueueEntry { task, tx } = entry;
                        let validity = task.get_validity();
                        tasks.push(task);
                        callbacks.push((validity, tx));
                    }

                    if !tasks.is_empty() {
                        let result = db.execute_tasks(&tasks)?;

                        if result {
                            for (validity, tx) in callbacks {
                                if let Err(e) = tx.send(LeasingResponse::Granted { validity }) {
                                    error!("Cannot send leasing result to client! ({:?})", e)
                                }
                            }
                        } else {
                            for (_, tx) in callbacks {
                                if let Err(e) = tx.send(LeasingResponse::Rejected) {
                                    error!("Cannot send leasing result to client! ({:?})", e)
                                }
                            }
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

        let task = match cache_result {
            CacheResult::Rejected => return Ok(LeasingResponse::Rejected),
            CacheResult::GrantedInsert {
                application_id,
                instance_id,
                validity,
            } => DatabaseTask::Insert {
                application_id,
                instance_id,
                validity,
            },
            CacheResult::GrantedUpdate {
                application_id,
                instance_id,
                validity,
            } => DatabaseTask::Update {
                application_id,
                instance_id,
                validity,
            },
        };

        let (tx, rx) = oneshot::channel();
        let entry = QueueEntry { task, tx };

        {
            let mut queue = self.queue.write().await;
            queue.push(entry);
        }

        self.notify.notify_one();
        Ok(rx.await?)
    }
}
