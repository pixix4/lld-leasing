use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task,
};

use crate::{
    context::{Context, LeasingResponse},
    env,
    utils::unpack_tcp_packet,
};

pub async fn start_server(context: Context) {
    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), *env::TCP_PORT))
        .await
        .unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let socket_context = context.clone();
        task::spawn(async move {
            process_socket_request(socket, socket_context).await;
        });
    }
}

async fn process_socket_request(mut socket: TcpStream, context: Context) {
    let mut packet = [0u8; 24];
    socket.read_exact(&mut packet).await.unwrap();
    let (instance_id, application_id, duration) = unpack_tcp_packet(packet);

    let rx = context
        .request_leasing(instance_id.clone(), application_id.clone(), duration)
        .await;

    let response = rx.await;
    match response {
        Err(e) => {
            eprintln!("Error while waiting for database result {}", e);
            socket.write_u8(50).await.unwrap();
        }
        Ok(LeasingResponse::Success {
            instance_id: _,
            application_id: _,
            validity: _,
        }) => {
            socket.write_u8(48).await.unwrap();
        }
        Ok(LeasingResponse::Error {
            instance_id: _,
            application_id: _,
        }) => {
            socket.write_u8(49).await.unwrap();
        }
    };
}
