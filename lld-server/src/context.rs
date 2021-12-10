use std::fmt::Debug;

use lld_common::get_current_time;

use crate::context_batching::ContextBatching;
use crate::context_naive::ContextNaive;
use crate::LldResult;

#[derive(Clone)]
pub enum Context {
    Naive(ContextNaive),
    Batching(ContextBatching),
}

impl Context {
    pub async fn run(&self) -> LldResult<()> {
        match self {
            Context::Naive(context) => context.run().await,
            Context::Batching(context) => context.run().await,
        }
    }

    pub async fn request_leasing(
        &self,
        application_id: String,
        instance_id: String,
        duration: u64,
    ) -> LldResult<LeasingResponse> {
        debug!(
            "Request leasing for {} with duration {}",
            application_id, duration
        );
        let now = get_current_time();

        match self {
            Context::Naive(context) => {
                context
                    .request_leasing(application_id, instance_id, duration, now)
                    .await
            }
            Context::Batching(context) => {
                context
                    .request_leasing(application_id, instance_id, duration, now)
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
