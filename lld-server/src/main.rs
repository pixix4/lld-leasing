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
        .get_matches();

    let http_port = value_t!(m, "http_port", u16).unwrap_or(3030);
    let tcp_port = value_t!(m, "tcp_port", u16).unwrap_or(3040);
    let mode = value_t!(m, "mode", LldMode).unwrap_or_default();

    info!("Initialze database");
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
    spawn(async move {
        http_api::start_server(http_api_context, http_port).await;
    });

    info!("Start tcp endpoint");
    let tcp_api_context = context.clone();
    spawn(async move {
        tcp_api::start_server(tcp_api_context, tcp_port).await;
    });

    info!("Start working queue");
    spawn(async move {
        context.run().await.unwrap();
    });

    tokio::signal::ctrl_c().await?;

    Ok(())
}
