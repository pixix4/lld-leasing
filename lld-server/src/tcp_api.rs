use std::{net::SocketAddr, time::Instant};

use lld_common::unpack_tcp_packet;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

use crate::context::{Context, LeasingResponse};

pub async fn start_server(context: Context, port: u16) {
    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), port))
        .await
        .unwrap();

    info!("Start tcp server at 0.0.0.0:{}", port);
    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let socket_context = context.clone();
        task::spawn(async move {
            process_socket_request(socket, addr, socket_context).await;
        });
    }
}

async fn process_socket_request(mut socket: TcpStream, addr: SocketAddr, context: Context) {
    let start = Instant::now();

    let mut packet = [0u8; 24];
    if let Err(e) = socket.read_exact(&mut packet).await {
        error!("Cannot recieve tcp request: {:?}", e);
        return;
    }
    let (application_id, instance_id, duration) = unpack_tcp_packet(packet);

    let response = context
        .request_leasing(application_id.clone(), instance_id.clone(), duration)
        .await;

    let duration = start.elapsed();
    info!("{} {:?} {}ms", addr, response, duration.as_millis());

    let send_result = match response {
        Ok(LeasingResponse::Granted { .. }) => socket.write_u8(48).await,
        Ok(LeasingResponse::Rejected) => socket.write_u8(49).await,
        Err(e) => {
            error!("Error while waiting for database result {:?}", e);
            socket.write_u8(50).await
        }
    };

    if let Err(e) = send_result {
        error!("Cannot send tcp response: {:?}", e);
    }
}
