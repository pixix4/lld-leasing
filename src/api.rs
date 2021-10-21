use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::context::Context;

#[derive(Debug, Deserialize, Serialize)]
pub struct RestLeasingRequest {
    id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum RestLeasingResponse {
    Success { id: String, validity: i64 },
    Error { id: String },
}

pub async fn start_server(context: Context) {
    let api = filters::leasing(context);
    let routes = api.with(warp::log("leasing"));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
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
        let rx = context.request_leasing(request.id.clone()).await;
        let response = rx.await;

        Ok(match response {
            Err(e) => {
                eprintln!("Error while waiting for database result {}", e);
                warp::reply::json(&RestLeasingResponse::Error { id: request.id })
            }
            Ok(LeasingResponse::Success { id, validity }) => {
                warp::reply::json(&RestLeasingResponse::Success { id, validity })
            }
            Ok(LeasingResponse::Error { id }) => {
                warp::reply::json(&RestLeasingResponse::Error { id })
            }
        })
    }
}
