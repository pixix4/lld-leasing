use std::pin::Pin;
use std::{net::SocketAddr, time::Instant};

use lld_common::unpack_tcp_packet;
use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task;
use tokio_openssl::SslStream;

use crate::context::{Context, LeasingResponse};

pub async fn start_server(context: Context, port: u16) {
    let listener = TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), port))
        .await
        .unwrap();

    info!("Start tcp server at 0.0.0.0:{}", port);
    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let mut acceptor = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).unwrap();
        acceptor.set_ca_file("certificates/root.crt").unwrap();
        acceptor
            .set_private_key_file("certificates/lld-server.key", SslFiletype::PEM)
            .unwrap();
        acceptor
            .set_certificate_chain_file("certificates/lld-server.crt")
            .unwrap();
        acceptor.check_private_key().unwrap();
        let acceptor = acceptor.build();

        let ssl = Ssl::new(acceptor.context()).unwrap();
        let mut stream = SslStream::new(ssl, socket).unwrap();

        Pin::new(&mut stream).accept().await.unwrap();

        let socket_context = context.clone();
        task::spawn(async move {
            process_socket_request(stream, addr, socket_context).await;
        });
    }
}

async fn process_socket_request<T>(mut socket: T, addr: SocketAddr, context: Context)
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let start = Instant::now();

    let mut packet = [0u8; 24];
    if let Err(e) = socket.read_exact(&mut packet).await {
        error!("Cannot receive tcp request: {:?}", e);
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
