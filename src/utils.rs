use std::time::{SystemTime, UNIX_EPOCH};

use hyper::{Body, Client, Method, Request};
use rand::{thread_rng, RngCore};

use crate::{api::RestLeasingResponse, env, LldResult};

pub async fn http_request_leasing(
    instance_id: &str,
    application_id: &str,
) -> LldResult<Option<i64>> {
    let client = Client::new();

    let req = Request::builder()
        .method(Method::POST)
        .uri(env::REQUEST_URI.as_str())
        .header("content-type", "application/json")
        .body(Body::from(format!(
            "{{\"instance_id\":\"{}\", \"application_id\":\"{}\"}}",
            instance_id, application_id
        )))?;

    let mut resp = client.request(req).await?;

    let bytes = hyper::body::to_bytes(resp.body_mut()).await?;
    let result = String::from_utf8(bytes.into_iter().collect())?;

    let response: RestLeasingResponse = serde_json::from_str(&result)?;

    Ok(match response {
        RestLeasingResponse::Success {
            instance_id: _,
            application_id: _,
            validity,
        } => Some(validity),
        RestLeasingResponse::Error {
            instance_id: _,
            application_id: _,
        } => None,
    })
}

pub fn generate_random_id<const T: usize>() -> String {
    let mut buffer = [0u8; T];
    thread_rng().fill_bytes(&mut buffer);
    base64::encode(&buffer)
}

pub fn get_current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}
