use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use hyper::{Body, Client, Method, Request};
use rand::{thread_rng, RngCore};
use tokio::net::TcpStream;

use crate::{
    env,
    http_api::{RestLeasingRequest, RestLeasingResponse},
    LldResult,
};

pub async fn http_request_leasing(
    instance_id: &str,
    application_id: &str,
    duration: u64,
) -> LldResult<Option<u64>> {
    let client = Client::new();

    let request = RestLeasingRequest {
        instance_id: instance_id.to_owned(),
        application_id: application_id.to_owned(),
        duration,
    };

    let req = Request::builder()
        .method(Method::POST)
        .uri(env::HTTP_REQUEST_URI.as_str())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request)?))?;

    let mut resp = client.request(req).await?;

    let bytes = hyper::body::to_bytes(resp.body_mut()).await?;
    let result = String::from_utf8(bytes.into_iter().collect())?;

    let response: RestLeasingResponse = serde_json::from_str(&result)?;

    Ok(match response {
        RestLeasingResponse::Granted { validity } => Some(validity),
        RestLeasingResponse::Rejected => None,
        RestLeasingResponse::Error => None,
    })
}

pub async fn tcp_request_leasing(
    instance_id: u64,
    application_id: u64,
    duration: u64,
) -> LldResult<Option<u64>> {
    let mut stream = TcpStream::connect(env::TCP_REQUEST_URI.as_str()).await?;

    let packet = pack_tcp_packet(instance_id, application_id, duration);
    tokio::io::AsyncWriteExt::write_all(&mut stream, &packet).await?;

    let result = tokio::io::AsyncReadExt::read_u8(&mut stream).await?;

    if result == 48 {
        let now = get_current_time();
        Ok(Some(duration + now))
    } else {
        Ok(None)
    }
}

pub fn generate_random_id<const T: usize>() -> String {
    let mut buffer = [0u8; T];
    thread_rng().fill_bytes(&mut buffer);
    base64::encode(&buffer)
}

pub fn generate_random_u64() -> u64 {
    thread_rng().next_u64()
}
pub fn generate_random_u8() -> u8 {
    thread_rng().next_u64() as u8
}

pub fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

pub fn unpack_tcp_packet(packet: [u8; 24]) -> (String, String, u64) {
    let instance_id = base64::encode(&packet[0..8]);
    let application_id = base64::encode(&packet[8..16]);
    let mut duration_slice: &[u8] = &packet[16..24];
    let duration = duration_slice.read_u64::<BigEndian>().unwrap();

    (instance_id, application_id, duration)
}

pub fn pack_tcp_packet(instance_id: u64, application_id: u64, duration: u64) -> [u8; 24] {
    let mut packet = [0u8; 24];
    let mut buffer = packet.as_mut();

    buffer.write_u64::<BigEndian>(instance_id).unwrap();
    buffer.write_u64::<BigEndian>(application_id).unwrap();
    buffer.write_u64::<BigEndian>(duration).unwrap();

    packet
}
