#[macro_use]
extern crate clap;
extern crate log;

mod benchmark;
mod docker;

use std::time::Duration;

use clap::{App, Arg};
use docker::DockerComposeFile;
use lld_common::{get_current_time, Environment, LldResult};
use log::info;
use tokio::time::sleep;

use crate::benchmark::start_concurrent_connections_round;

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum LldContainer {
        NativeSqliteNaive,
        NativeSqliteCaching,
        NativeSqliteBatching,
        NativeDqlite,
        NativeDqliteNaive,
        NativeSqliteOptimized,
        SconeDqlite,
    }
}

impl Into<DockerComposeFile> for LldContainer {
    fn into(self) -> DockerComposeFile {
        match self {
            LldContainer::NativeSqliteNaive => DockerComposeFile::NativeSqliteNaive,
            LldContainer::NativeSqliteCaching => DockerComposeFile::NativeSqliteCaching,
            LldContainer::NativeSqliteBatching => DockerComposeFile::NativeSqliteBatching,
            LldContainer::NativeDqlite => DockerComposeFile::NativeDqlite,
            LldContainer::NativeDqliteNaive => DockerComposeFile::NativeDqliteNaive,
            LldContainer::NativeSqliteOptimized => DockerComposeFile::NativeSqliteOptimized,
            LldContainer::SconeDqlite => DockerComposeFile::SconeDqlite,
        }
    }
}

impl LldContainer {
    async fn before_sleep(self) {
        match self {
            LldContainer::NativeSqliteNaive => sleep(Duration::from_millis(3000)).await,
            LldContainer::NativeSqliteCaching => sleep(Duration::from_millis(3000)).await,
            LldContainer::NativeSqliteBatching => sleep(Duration::from_millis(3000)).await,
            LldContainer::NativeSqliteOptimized => sleep(Duration::from_millis(3000)).await,
            LldContainer::NativeDqlite => sleep(Duration::from_millis(15000)).await,
            LldContainer::NativeDqliteNaive => sleep(Duration::from_millis(15000)).await,
            LldContainer::SconeDqlite => sleep(Duration::from_millis(45_000)).await,
        }
    }
    async fn after_sleep(self) {
        match self {
            LldContainer::NativeSqliteNaive => sleep(Duration::from_millis(800)).await,
            LldContainer::NativeSqliteCaching => sleep(Duration::from_millis(800)).await,
            LldContainer::NativeSqliteBatching => sleep(Duration::from_millis(800)).await,
            LldContainer::NativeSqliteOptimized => sleep(Duration::from_millis(800)).await,
            LldContainer::NativeDqlite => sleep(Duration::from_millis(1000)).await,
            LldContainer::NativeDqliteNaive => sleep(Duration::from_millis(1000)).await,
            LldContainer::SconeDqlite => sleep(Duration::from_millis(10_000)).await,
        }
    }
}

async fn start_step(
    environment: &Environment,
    container: LldContainer,
    count: usize,
    repeat: usize,
    duration: usize,
) -> LldResult<()> {
    let name = container.to_string();
    for round in 0..repeat {
        info!(
            "Start round {} of {}: {}, {} clients, {} ms",
            round + 1,
            repeat,
            name,
            count * 2,
            duration
        );

        // let mode_string = mode.to_string();
        // let env = [("LLD_MODE", mode_string.as_str())];
        let file: DockerComposeFile = container.into();
        file.up().await?;

        container.before_sleep().await;

        let stop_at = get_current_time() + (duration as u64);
        let result = start_concurrent_connections_round(environment, count, stop_at).await;

        sleep(Duration::from_millis(200)).await;

        file.down().await?;

        container.after_sleep().await;

        match result {
            Ok(result) => {
                let granted_avg = result.granted_time as f64 / result.granted_count as f64;
                let rejected_avg = result.rejected_time as f64 / result.rejected_count as f64;
                let timeout_avg = result.timeout_time as f64 / result.timeout_count as f64;
                let error_avg = result.error_time as f64 / result.error_count as f64;

                println!(
                    "{},{},{},{},{},{},{},{},{},{}",
                    name,
                    count * 2,
                    if granted_avg.is_nan() {
                        0.0
                    } else {
                        granted_avg
                    },
                    if rejected_avg.is_nan() {
                        0.0
                    } else {
                        rejected_avg
                    },
                    if timeout_avg.is_nan() {
                        0.0
                    } else {
                        timeout_avg
                    },
                    if error_avg.is_nan() { 0.0 } else { error_avg },
                    result.granted_count,
                    result.rejected_count,
                    result.timeout_count,
                    result.error_count,
                );
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> LldResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let m = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("http_uri")
                .long("http_uri")
                .env("LLD_HTTP_URI"),
        )
        .arg(Arg::with_name("tcp_uri").long("tcp_uri").env("LLD_TCP_URI"))
        .arg(
            Arg::with_name("repeat")
                .short("r")
                .long("repeat")
                .env("LLD_REPEAT"),
        )
        .arg(Arg::with_name("max").short("m").long("max").env("LLD_MAX"))
        .arg(
            Arg::with_name("duration")
                .short("d")
                .long("duration")
                .env("LLD_DURATION"),
        )
        .arg(
            Arg::with_name("container")
                .short("c")
                .long("container")
                .env("LLD_CONTAINER")
                .possible_values(&LldContainer::variants()),
        )
        .arg(
            Arg::with_name("ssl_cert_file")
                .long("ssl_cert_file")
                .env("LLD_CERT_FILE"),
        )
        .get_matches();

    let ssl_cert_file = m.value_of("ssl_cert_file").unwrap_or("cacert.pem");

    let tcp_uri = m.value_of("tcp_uri").unwrap_or("127.0.0.1:3040");

    let repeat = value_t!(m, "repeat", usize).unwrap_or(3);
    let max = value_t!(m, "max", usize).unwrap_or(8);
    let duration = value_t!(m, "duration", usize).unwrap_or(5000);

    println!("type,count,granted_avg,rejected_avg,timeout_avg,error_avg,granted_count,rejected_count,timeout_count,error_count");

    for container in [
        // LldContainer::NativeSqliteNaive,
        // LldContainer::NativeSqliteCaching,
        // LldContainer::NativeSqliteBatching,
        LldContainer::NativeDqlite,
        LldContainer::NativeDqliteNaive,
        // LldContainer::NativeSqliteOptimized,
        // LldContainer::SconeDqlite,
    ] {
        let ssl_cert_file = match container {
            LldContainer::SconeDqlite => {
                info!("Client will use ssl encryption");
                Some(ssl_cert_file)
            }
            _ => {
                info!("Client will use plain text");
                None
            }
        };
        let http_uri = if ssl_cert_file.is_some() {
            m.value_of("http_uri")
                .unwrap_or("https://localhost:3030/request")
        } else {
            m.value_of("http_uri")
                .unwrap_or("http://localhost:3030/request")
        };

        let environment = Environment {
            http_request_uri: http_uri.to_owned(),
            tcp_request_uri: tcp_uri.to_owned(),
            ssl_cert_file: ssl_cert_file.map(str::to_string),
        };

        let mut count = 1;
        for _ in 1..max {
            start_step(&environment, container, count, repeat, duration).await?;

            count *= 2;
        }
    }

    Ok(())
}
