#[macro_use]
extern crate log;

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

use clap::Parser;
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value_t = 3030)]
    http_port: u16,
    #[clap(long, default_value_t = 3040)]
    tcp_port: u16,
    #[clap(long)]
    sqlite_optimization: bool,
    #[clap(long, default_value_t=LldMode::Batching)]
    mode: LldMode,
    #[clap(long, default_value_t=String::from("certificates/lld-server.key"))]
    ssl_key_file: String,
    #[clap(long, default_value_t=String::from("certificates/lld-server.crt"))]
    ssl_cert_file: String,
}

#[tokio::main]
async fn main() -> LldResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Args::parse();

    let ssl_context = if std::path::Path::new(&args.ssl_key_file).exists()
        && std::path::Path::new(&args.ssl_cert_file).exists()
    {
        info!("Server will use ssl encryption");
        Some(SslContext {
            cert_file: args.ssl_cert_file,
            key_file: args.ssl_key_file,
        })
    } else {
        info!("Server will use plain text");
        None
    };

    info!("Initialize database");
    let db = Database::open(args.sqlite_optimization)?;
    db.init()?;

    let context = match args.mode {
        LldMode::Naive => {
            info!("Naive");
            Context::Naive(ContextNaive::new_without_cache(db)?)
        }
        LldMode::NaiveCaching => {
            info!("NaiveCaching");
            Context::Naive(ContextNaive::new(db)?)
        }
        LldMode::Batching => {
            info!("Batching");
            Context::Batching(ContextBatching::new(db)?)
        }
    };

    info!("Start http endpoint");
    let http_api_context = context.clone();
    let http_ssl_context = ssl_context.clone();
    let http_port = args.http_port;
    spawn(async move {
        http_api::start_server(http_api_context, http_port, http_ssl_context).await;
    });

    info!("Start tcp endpoint");
    let tcp_api_context = context.clone();
    let tcp_port = args.tcp_port;
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
