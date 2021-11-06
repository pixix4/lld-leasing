use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{database::Database, LldResult};

pub type CacheMap = HashMap<String, (String, u64)>;

#[derive(Debug, Clone)]
pub struct ContextCache {
    cache: Arc<RwLock<CacheMap>>,
}

#[derive(Debug)]
pub enum CacheResult {
    Rejected,
    GrantedInsert {
        application_id: String,
        instance_id: String,
        validity: u64,
    },
    GrantedUpdate {
        application_id: String,
        instance_id: String,
        validity: u64,
    },
}

impl ContextCache {
    pub fn new(db: &Database) -> LldResult<Self> {
        let cache = db.build_cache()?;

        Ok(Self {
            cache: Arc::new(RwLock::new(cache)),
        })
    }

    pub async fn request_leasing(
        &self,
        application_id: String,
        instance_id: String,
        duration: u64,
        now: u64,
    ) -> LldResult<CacheResult> {
        let found = {
            let cache = self.cache.read().await;
            cache.get(&application_id).cloned()
        };

        Ok(match found {
            Some((leased_instance_id, validity)) => {
                if validity > now && leased_instance_id != instance_id {
                    CacheResult::Rejected
                } else {
                    {
                        let mut cache = self.cache.write().await;
                        cache.insert(
                            application_id.to_owned(),
                            (instance_id.to_owned(), now + duration),
                        );
                    }
                    CacheResult::GrantedUpdate {
                        application_id,
                        instance_id,
                        validity: now + duration,
                    }
                }
            }
            None => {
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(
                        application_id.to_owned(),
                        (instance_id.to_owned(), now + duration),
                    );
                }
                CacheResult::GrantedInsert {
                    application_id,
                    instance_id,
                    validity: now + duration,
                }
            }
        })
    }
}
