#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod cache;
mod context;
mod context_batching;
mod context_naive;
mod database;
mod http_api;
mod tcp_api;

#[cfg(feature = "dqlite")]
mod dqlite;
#[cfg(not(feature = "dqlite"))]
mod sqlite;

use clap::{App, Arg};
use context::Context;
use context_batching::ContextBatching;
use context_naive::ContextNaive;
use database::Database;
use lld_common::{LldMode, LldResult};

use tokio::spawn;

#[derive(Debug, Clone)]
pub struct SslContext {
    pub cert_file: String,
    pub key_file: String,
}

#[tokio::main]
async fn main() -> LldResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let m = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("http_port")
                .long("http_port")
                .env("LLD_HTTP_PORT"),
        )
        .arg(
            Arg::with_name("tcp_port")
                .long("tcp_port")
                .env("LLD_TCP_PORT"),
        )
        .arg(
            Arg::with_name("mode")
                .env("LLD_MODE")
                .possible_values(&LldMode::variants()),
        )
        .arg(
            Arg::with_name("ssl_key_file")
                .long("ssl_key_file")
                .env("LLD_KEY_FILE"),
        )
        .arg(
            Arg::with_name("ssl_cert_file")
                .long("ssl_cert_file")
                .env("LLD_CERT_FILE"),
        )
        .get_matches();

    let http_port = value_t!(m, "http_port", u16).unwrap_or(3030);
    let tcp_port = value_t!(m, "tcp_port", u16).unwrap_or(3040);
    let mode = value_t!(m, "mode", LldMode).unwrap_or_default();

    let ssl_key_file = value_t!(m, "ssl_key_file", String)
        .unwrap_or_else(|_| "certificates/lld-server.key".to_owned());
    let ssl_cert_file = value_t!(m, "ssl_cert_file", String)
        .unwrap_or_else(|_| "certificates/lld-server.crt".to_owned());

    let ssl_context = if std::path::Path::new(&ssl_key_file).exists()
        && std::path::Path::new(&ssl_cert_file).exists()
    {
        info!("Server will use ssl encryption");
        Some(SslContext {
            cert_file: ssl_cert_file,
            key_file: ssl_key_file,
        })
    } else {
        info!("Server will use plain text");
        None
    };

    info!("Initialize database");
    let db = Database::open()?;
    db.init()?;

    info!("Start context in {:?} mode", mode);
    let context = match mode {
        LldMode::Naive => Context::Naive(ContextNaive::new_without_cache(db)?),
        LldMode::NaiveCaching => Context::Naive(ContextNaive::new(db)?),
        LldMode::Batching => Context::Batching(ContextBatching::new(db)?),
    };

    info!("Start http endpoint");
    let http_api_context = context.clone();
    let http_ssl_context = ssl_context.clone();
    spawn(async move {
        http_api::start_server(http_api_context, http_port, http_ssl_context).await;
    });

    info!("Start tcp endpoint");
    let tcp_api_context = context.clone();
    spawn(async move {
        tcp_api::start_server(tcp_api_context, tcp_port, ssl_context).await;
    });

    info!("Start working queue");
    spawn(async move {
        context.run().await.unwrap();
    });

    tokio::signal::ctrl_c().await?;

    Ok(())
}
