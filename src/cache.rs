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

    pub fn to_cache_result(
        application_id: String,
        instance_id: String,
        duration: u64,
        now: u64,
        result: Option<(String, u64)>,
    ) -> CacheResult {
        match result {
            Some((leased_instance_id, validity)) => {
                if validity > now && leased_instance_id != instance_id {
                    CacheResult::Rejected
                } else {
                    CacheResult::GrantedUpdate {
                        application_id,
                        instance_id,
                        validity: now + duration,
                    }
                }
            }
            None => CacheResult::GrantedInsert {
                application_id,
                instance_id,
                validity: now + duration,
            },
        }
    }

    pub async fn request_leasing(
        &self,
        application_id: String,
        instance_id: String,
        duration: u64,
        now: u64,
    ) -> LldResult<CacheResult> {
        let result = {
            let cache = self.cache.read().await;
            cache.get(&application_id).cloned()
        };

        let cache_result =
            ContextCache::to_cache_result(application_id, instance_id, duration, now, result);

        let cache_result = match cache_result {
            CacheResult::Rejected => CacheResult::Rejected,
            CacheResult::GrantedInsert {
                application_id,
                instance_id,
                validity,
            } => {
                let mut cache = self.cache.write().await;
                if cache.get(&application_id).is_some() {
                    CacheResult::Rejected
                } else {
                    cache.insert(
                        application_id.to_owned(),
                        (instance_id.to_owned(), now + duration),
                    );
                    CacheResult::GrantedInsert {
                        application_id,
                        instance_id,
                        validity,
                    }
                }
            }
            CacheResult::GrantedUpdate {
                application_id,
                instance_id,
                validity,
            } => {
                let mut cache = self.cache.write().await;
                if cache.get(&application_id).unwrap().0.as_str() != instance_id {
                    CacheResult::Rejected
                } else {
                    cache.insert(
                        application_id.to_owned(),
                        (instance_id.to_owned(), now + duration),
                    );
                    CacheResult::GrantedUpdate {
                        application_id,
                        instance_id,
                        validity,
                    }
                }
            }
        };

        Ok(cache_result)
    }
}
