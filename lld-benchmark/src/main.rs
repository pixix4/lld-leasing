#[macro_use]
extern crate clap;
extern crate log;

mod benchmark;
mod docker;

use std::time::Duration;

use clap::{App, Arg, SubCommand};
use docker::{ContainerRef, DockerImage};
use lld_common::{get_current_time, Environment, LldMode, LldResult};
use log::info;
use tokio::time::sleep;

use crate::benchmark::start_concurrent_connections_round;

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum LldContainer {
        NativeSqlite,
        NativeDqlite,
        SconeSqlite,
        SconeDqlite,
    }
}

impl Default for LldContainer {
    fn default() -> Self {
        Self::NativeSqlite
    }
}

async fn setup_images(container: Option<LldContainer>, force_build: bool) -> LldResult<()> {
    match container {
        Some(container) => match container {
            LldContainer::NativeSqlite => {
                DockerImage::LldNativeSqlite
                    .build_image_if_not_exists(force_build)
                    .await?;
            }
            LldContainer::NativeDqlite => {
                DockerImage::DqliteServer
                    .build_image_if_not_exists(force_build)
                    .await?;
                DockerImage::LldNativeDqlite
                    .build_image_if_not_exists(force_build)
                    .await?;
            }
            LldContainer::SconeSqlite => {
                DockerImage::LldSconeSqlite
                    .build_image_if_not_exists(force_build)
                    .await?;
            }
            LldContainer::SconeDqlite => {
                DockerImage::DqliteServer
                    .build_image_if_not_exists(force_build)
                    .await?;
                DockerImage::LldSconeDqlite
                    .build_image_if_not_exists(force_build)
                    .await?;
            }
        },
        None => {
            DockerImage::DqliteServer
                .build_image_if_not_exists(force_build)
                .await?;
            DockerImage::LldNativeDqlite
                .build_image_if_not_exists(force_build)
                .await?;
            DockerImage::LldNativeSqlite
                .build_image_if_not_exists(force_build)
                .await?;
            DockerImage::LldSconeDqlite
                .build_image_if_not_exists(force_build)
                .await?;
            DockerImage::LldSconeSqlite
                .build_image_if_not_exists(force_build)
                .await?;
        }
    }

    Ok(())
}

async fn start_round(mode: LldMode, container: LldContainer) -> LldResult<Vec<ContainerRef>> {
    let mut containers = Vec::new();
    let mode_string = mode.to_string();
    let env = [("LLD_MODE", mode_string.as_str())];

    match container {
        LldContainer::NativeSqlite => {
            containers.push(DockerImage::LldNativeSqlite.start_container(&env).await?);
        }
        LldContainer::NativeDqlite => {
            containers.push(DockerImage::DqliteServer.start_container(&[]).await?);
            containers.push(DockerImage::LldNativeDqlite.start_container(&env).await?);
        }
        LldContainer::SconeSqlite => {
            containers.push(DockerImage::LldSconeSqlite.start_container(&env).await?);
        }
        LldContainer::SconeDqlite => {
            containers.push(DockerImage::DqliteServer.start_container(&[]).await?);
            containers.push(DockerImage::LldSconeDqlite.start_container(&env).await?);
        }
    }

    Ok(containers)
}

async fn stop_round(containers: Vec<ContainerRef>) -> LldResult<()> {
    for container_ref in containers {
        container_ref.stop_container().await?;
    }
    Ok(())
}

async fn start_step(
    environment: &Environment,
    mode: LldMode,
    container: LldContainer,
    count: usize,
    repeat: usize,
    duration: usize,
) -> LldResult<()> {
    let name = mode.to_string();
    for round in 0..repeat {
        info!(
            "Start round {} of {}: {}, {} clients, {} ms",
            round + 1,
            repeat,
            name,
            count * 2,
            duration
        );

        let containers = start_round(mode, container).await?;

        sleep(Duration::from_millis(100)).await;

        let stop_at = get_current_time() + (duration as u64);
        let result = start_concurrent_connections_round(environment, count, stop_at).await;

        sleep(Duration::from_millis(200)).await;

        stop_round(containers).await?;

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
            Arg::with_name("force_build")
                .long("build")
                .env("LLD_FORCE_BUILD"),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about("Builds the docker images without running the tests")
                .arg(
                    Arg::with_name("image")
                        .help("Name of the docker image. Build all if no name is specified."),
                )
                .arg(
                    Arg::with_name("force_build")
                        .long("build")
                        .env("LLD_FORCE_BUILD"),
                ),
        )
        .get_matches();

    if let Some(m) = m.subcommand_matches("build") {
        let force_build = m.is_present("force_build");

        setup_images(None, force_build).await?;

        return Ok(());
    }

    let http_uri = m
        .value_of("http_uri")
        .unwrap_or("http://localhost:3030/request");
    let tcp_uri = m.value_of("tcp_uri").unwrap_or("127.0.0.1:3040");

    let environment = Environment {
        http_request_uri: http_uri.to_owned(),
        tcp_request_uri: tcp_uri.to_owned(),
    };

    let repeat = value_t!(m, "repeat", usize).unwrap_or(1);
    let max = value_t!(m, "max", usize).unwrap_or(3);
    let duration = value_t!(m, "duration", usize).unwrap_or(5000);
    let force_build = m.is_present("force_build");
    let container = value_t!(m, "container", LldContainer).unwrap_or_default();

    setup_images(Some(container), force_build).await?;

    let mut count = 1;

    println!("type,count,granted_avg,rejected_avg,timeout_avg,error_avg,granted_count,rejected_count,timeout_count,error_count");

    for _ in 1..max {
        for mode in [LldMode::Naive, LldMode::NaiveCaching, LldMode::Batching] {
            start_step(&environment, mode, container, count, repeat, duration).await?;
        }

        count *= 2;
    }

    Ok(())
}
