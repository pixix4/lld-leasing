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
    cache: ContextCache,
}

impl ContextNaive {
    #[allow(clippy::new_without_default)]
    pub fn new(db: Database) -> LldResult<Self> {
        let cache = ContextCache::new(&db)?;
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
        let cache_result = self
            .cache
            .request_leasing(application_id, instance_id, duration, now)
            .await?;

        let leasing_result = match cache_result {
            CacheResult::Rejected => LeasingResponse::Rejected,
            CacheResult::GrantedInsert {
                application_id,
                instance_id,
                validity,
            } => {
                let _lock = self.lock.lock().await;
                let db = Database::open()?;
                db.insert_leasing(&application_id, &instance_id, validity)?;

                LeasingResponse::Granted { validity }
            }
            CacheResult::GrantedUpdate {
                application_id,
                instance_id,
                validity,
            } => {
                let _lock = self.lock.lock().await;
                let db = Database::open()?;
                db.update_leasing(&application_id, &instance_id, validity)?;

                LeasingResponse::Granted { validity }
            }
        };

        Ok(leasing_result)
    }
}
