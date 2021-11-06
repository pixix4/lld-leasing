use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{context::Context, env};

#[derive(Debug, Deserialize, Serialize)]
pub struct RestLeasingRequest {
    pub application_id: String,
    pub instance_id: String,
    pub duration: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum RestLeasingResponse {
    Granted { validity: u64 },
    Rejected,
    Error,
}

pub async fn start_server(context: Context) {
    let api = filters::leasing(context);
    let routes = api.with(warp::log("http_api"));
    warp::serve(routes)
        .run(([0, 0, 0, 0], *env::HTTP_PORT))
        .await;
}

mod filters {
    use super::{handlers, RestLeasingRequest};
    use crate::context::Context;
    use warp::Filter;

    pub fn leasing(
        context: Context,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        request_leasing(context)
    }

    pub fn request_leasing(
        context: Context,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("request")
            .and(warp::post())
            .and(json_body())
            .and(with_context(context))
            .and_then(handlers::request_leasing)
    }

    fn with_context(
        context: Context,
    ) -> impl Filter<Extract = (Context,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || context.clone())
    }

    fn json_body() -> impl Filter<Extract = (RestLeasingRequest,), Error = warp::Rejection> + Clone
    {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

mod handlers {
    use crate::context::{Context, LeasingResponse};
    use std::convert::Infallible;

    use super::{RestLeasingRequest, RestLeasingResponse};

    pub async fn request_leasing(
        request: RestLeasingRequest,
        context: Context,
    ) -> Result<impl warp::Reply, Infallible> {
        let response = context
            .request_leasing(
                request.application_id.clone(),
                request.instance_id.clone(),
                request.duration,
            )
            .await;

        Ok(match response {
            Ok(LeasingResponse::Granted { validity }) => {
                warp::reply::json(&RestLeasingResponse::Granted { validity })
            }
            Ok(LeasingResponse::Rejected) => warp::reply::json(&RestLeasingResponse::Rejected),
            Err(e) => {
                error!("Error while waiting for database result {:?}", e);
                warp::reply::json(&RestLeasingResponse::Error)
            }
        })
    }
}
