use std::{fmt::Debug, sync::Arc};

use tokio::sync::{oneshot, Notify, RwLock};

use crate::{
    cache::{CacheResult, ContextCache},
    context::LeasingResponse,
    database::{Database, DatabaseTask},
    LldResult,
};

#[derive(Debug)]
pub enum QueueEntry {
    Cached {
        task: DatabaseTask,
        tx: oneshot::Sender<LeasingResponse>,
    },
    Default {
        application_id: String,
        instance_id: String,
        duration: u64,
        now: u64,
        tx: oneshot::Sender<LeasingResponse>,
    },
}

#[derive(Debug, Clone)]
pub struct ContextBatching {
    queue: Arc<RwLock<Vec<QueueEntry>>>,
    notify: Arc<Notify>,
    cache: Option<ContextCache>,
}

impl ContextBatching {
    #[allow(clippy::new_without_default)]
    pub fn new(db: Database, disable_cache: bool) -> LldResult<Self> {
        let cache = if disable_cache {
            None
        } else {
            Some(ContextCache::new(&db)?)
        };
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
        let db = Database::open()?;
        loop {
            match self.check_tasks().await {
                Some(entries) => {
                    let mut tasks = Vec::<DatabaseTask>::with_capacity(entries.len());
                    let mut callbacks =
                        Vec::<(u64, oneshot::Sender<LeasingResponse>)>::with_capacity(
                            entries.len(),
                        );

                    for entry in entries {
                        match entry {
                            QueueEntry::Cached { task, tx } => {
                                let validity = task.get_validity();
                                tasks.push(task);
                                callbacks.push((validity, tx));
                            }
                            QueueEntry::Default {
                                application_id,
                                instance_id,
                                duration,
                                now,
                                tx,
                            } => {
                                let result = db.query_leasing(&application_id)?;
                                let cache_result = ContextCache::to_cache_result(
                                    application_id,
                                    instance_id,
                                    duration,
                                    now,
                                    result,
                                );

                                let leasing_result = match cache_result {
                                    CacheResult::Rejected => LeasingResponse::Rejected,
                                    CacheResult::GrantedInsert {
                                        application_id,
                                        instance_id,
                                        validity,
                                    } => {
                                        db.insert_leasing(&application_id, &instance_id, validity)?;
                                        LeasingResponse::Granted { validity }
                                    }
                                    CacheResult::GrantedUpdate {
                                        application_id,
                                        instance_id,
                                        validity,
                                    } => {
                                        db.update_leasing(&application_id, &instance_id, validity)?;
                                        LeasingResponse::Granted { validity }
                                    }
                                };

                                if let Err(e) = tx.send(leasing_result) {
                                    error!("Cannot send leasing result to client! ({:?})", e)
                                }
                            }
                        }
                    }

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
        let (tx, rx) = oneshot::channel();
        let entry = match &self.cache {
            Some(cache) => {
                let cache_result = cache
                    .request_leasing(application_id, instance_id, duration, now)
                    .await?;

                QueueEntry::Cached {
                    task: match cache_result {
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
                    },
                    tx,
                }
            }
            None => QueueEntry::Default {
                application_id,
                instance_id,
                duration,
                now,
                tx,
            },
        };

        {
            let mut queue = self.queue.write().await;
            queue.push(entry);
        }
        self.notify.notify_one();
        Ok(rx.await?)
    }
}
