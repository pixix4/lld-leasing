use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::error;
use openssl::ssl::{SslConnector, SslMethod};
use rand::{thread_rng, RngCore};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

use crate::{LldError, LldResult};

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

#[derive(Debug, Clone)]
pub struct Environment {
    pub http_request_uri: String,
    pub tcp_request_uri: String,
    pub ssl_cert_file: Option<String>,
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum LldMode {
        Naive,
        NaiveCaching,
        Batching,
    }
}

impl Default for LldMode {
    fn default() -> Self {
        Self::Batching
    }
}

pub fn http_request_client(certificate_file: Option<&str>) -> LldResult<Client> {
    if let Some(certificate_file) = certificate_file {
        let cert = std::fs::read(certificate_file)?;
        let cert = reqwest::Certificate::from_pem(&cert)?;

        let client = reqwest::Client::builder()
            .tls_built_in_root_certs(false)
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()?;

        Ok(client)
    } else {
        let client = reqwest::Client::builder().build()?;

        Ok(client)
    }
}

pub async fn http_request_leasing(
    client: &Client,
    environment: &Environment,
    application_id: &str,
    instance_id: &str,
    duration: u64,
) -> LldResult<Option<u64>> {
    let request = RestLeasingRequest {
        application_id: application_id.to_owned(),
        instance_id: instance_id.to_owned(),
        duration,
    };

    let response = client
        .post(&environment.http_request_uri)
        .json(&request)
        .send()
        .await?
        .json::<RestLeasingResponse>()
        .await?;

    Ok(match response {
        RestLeasingResponse::Granted { validity } => Some(validity),
        RestLeasingResponse::Rejected => None,
        RestLeasingResponse::Error => {
            error!("Receive error response!");
            None
        }
    })
}

async fn tcp_request_leasing_socket<T>(
    mut stream: T,
    application_id: u64,
    instance_id: u64,
    duration: u64,
) -> LldResult<Option<u64>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let packet = pack_tcp_packet(application_id, instance_id, duration);
    tokio::io::AsyncWriteExt::write_all(&mut stream, &packet)
        .await
        .map_err(|error| {
            LldError::WrappedError(
                "tcp_request_leasing - write_all error",
                format!("{}", error),
            )
        })?;

    let result = tokio::io::AsyncReadExt::read_u8(&mut stream)
        .await
        .map_err(|error| {
            LldError::WrappedError("tcp_request_leasing - read_u8 error", format!("{}", error))
        })?;

    if result == 48 {
        let now = get_current_time();
        Ok(Some(duration + now))
    } else {
        Ok(None)
    }
}

pub async fn tcp_request_leasing(
    environment: &Environment,
    application_id: u64,
    instance_id: u64,
    duration: u64,
) -> LldResult<Option<u64>> {
    let stream = TcpStream::connect(&environment.tcp_request_uri)
        .await
        .map_err(|error| {
            LldError::WrappedError("tcp_request_leasing - connect error", format!("{}", error))
        })?;

    if let Some(ref certificate_file) = environment.ssl_cert_file {
        let mut connector = SslConnector::builder(SslMethod::tls())?;
        connector.set_ca_file(certificate_file)?;
        let ssl = connector.build().configure()?.into_ssl("api")?;

        let mut stream = SslStream::new(ssl, stream)?;

        Pin::new(&mut stream).connect().await?;

        tcp_request_leasing_socket(stream, application_id, instance_id, duration).await
    } else {
        tcp_request_leasing_socket(stream, application_id, instance_id, duration).await
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
    let application_id = base64::encode(&packet[0..8]);
    let instance_id = base64::encode(&packet[8..16]);
    let mut duration_slice: &[u8] = &packet[16..24];
    let duration = duration_slice.read_u64::<BigEndian>().unwrap_or(0);

    (application_id, instance_id, duration)
}

pub fn pack_tcp_packet(application_id: u64, instance_id: u64, duration: u64) -> [u8; 24] {
    let mut packet = [0u8; 24];
    let mut buffer = packet.as_mut();

    buffer
        .write_u64::<BigEndian>(application_id)
        .expect("Cannot write `application_id` to a tcp packet!");
    buffer
        .write_u64::<BigEndian>(instance_id)
        .expect("Cannot write `instance_id` to a tcp packet!");
    buffer
        .write_u64::<BigEndian>(duration)
        .expect("Cannot write `duration` to a tcp packet!");

    packet
}
