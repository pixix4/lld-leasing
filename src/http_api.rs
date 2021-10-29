use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{context::Context, env};

#[derive(Debug, Deserialize, Serialize)]
pub struct RestLeasingRequest {
    instance_id: String,
    application_id: String,
    duration: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum RestLeasingResponse {
    Success {
        instance_id: String,
        application_id: String,
        validity: u64,
    },
    Error {
        instance_id: String,
        application_id: String,
    },
}

pub async fn start_server(context: Context) {
    let api = filters::leasing(context);
    let routes = api.with(warp::log("leasing"));
    warp::serve(routes)
        .run(([0, 0, 0, 0], *env::HTTP_PORT))
        .await;
}

mod filters {
    use crate::context::Context;

    use super::{handlers, RestLeasingRequest};
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
        let rx = context
            .request_leasing(
                request.instance_id.clone(),
                request.application_id.clone(),
                request.duration,
            )
            .await;
        let response = rx.await;

        Ok(match response {
            Err(e) => {
                eprintln!("Error while waiting for database result {}", e);
                warp::reply::json(&RestLeasingResponse::Error {
                    instance_id: request.instance_id,
                    application_id: request.application_id,
                })
            }
            Ok(LeasingResponse::Success {
                instance_id,
                application_id,
                validity,
            }) => warp::reply::json(&RestLeasingResponse::Success {
                instance_id,
                application_id,
                validity,
            }),
            Ok(LeasingResponse::Error {
                instance_id,
                application_id,
            }) => warp::reply::json(&RestLeasingResponse::Error {
                instance_id,
                application_id,
            }),
        })
    }
}
