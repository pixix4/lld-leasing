use std::{net::SocketAddr, time::Instant};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task,
};

use crate::{
    context::{LeasingResponse, LldContext},
    env,
    utils::unpack_tcp_packet,
};

pub async fn start_server(context: LldContext) {
    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), *env::TCP_PORT))
        .await
        .unwrap();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let socket_context = context.clone();
        task::spawn(async move {
            process_socket_request(socket, addr, socket_context).await;
        });
    }
}

async fn process_socket_request(mut socket: TcpStream, addr: SocketAddr, context: LldContext) {
    let start = Instant::now();

    let mut packet = [0u8; 24];
    socket.read_exact(&mut packet).await.unwrap();
    let (instance_id, application_id, duration) = unpack_tcp_packet(packet);

    let response = context
        .request_leasing(instance_id.clone(), application_id.clone(), duration)
        .await;

    let duration = start.elapsed();
    info!("{} {:?} {}ms", addr, response, duration.as_millis());

    match response {
        Ok(LeasingResponse::Granted { .. }) => {
            socket.write_u8(48).await.unwrap();
        }
        Ok(LeasingResponse::Rejected) => {
            socket.write_u8(49).await.unwrap();
        }
        Err(e) => {
            error!("Error while waiting for database result {:?}", e);
            socket.write_u8(50).await.unwrap();
        }
    };
}
