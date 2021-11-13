use std::{fmt::Debug, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    cache::{CacheResult, ContextCache},
    context::LeasingResponse,
    database::Database,
    LldResult,
};

#[derive(Debug, Clone)]
pub struct ContextNaive {
    lock: Arc<Mutex<()>>,
    cache: Option<ContextCache>,
}

impl ContextNaive {
    #[allow(clippy::new_without_default)]
    pub fn new(db: Database, disable_cache: bool) -> LldResult<Self> {
        let cache = if disable_cache {
            None
        } else {
            Some(ContextCache::new(&db)?)
        };
        Ok(Self {
            lock: Arc::new(Mutex::new(())),
            cache,
        })
    }

    pub async fn run(&self) -> LldResult<()> {
        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    pub async fn request_leasing(
        &self,
        application_id: String,
        instance_id: String,
        duration: u64,
        now: u64,
    ) -> LldResult<LeasingResponse> {
        let cache_result = match &self.cache {
            Some(cache) => Some(
                cache
                    .request_leasing(application_id.clone(), instance_id.clone(), duration, now)
                    .await?,
            ),
            None => None,
        };

        if let Some(CacheResult::Rejected) = cache_result {
            return Ok(LeasingResponse::Rejected);
        }

        let _lock = self.lock.lock().await;
        let db = Database::open()?;
        let cache_result = if let Some(cache_result) = cache_result {
            cache_result
        } else {
            let result = db.query_leasing(&application_id)?;
            ContextCache::to_cache_result(application_id, instance_id, duration, now, result)
        };

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

        Ok(leasing_result)
    }
}
