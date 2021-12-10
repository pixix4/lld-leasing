#[macro_use]
extern crate clap;
extern crate log;

mod benchmark;
mod docker;

use std::{process::exit, time::Duration};

use clap::{App, Arg};
use lld_common::{get_current_time, Environment, LldMode, LldResult};
use log::error;
use tokio::time::sleep;

use crate::benchmark::start_concurrent_connections_round;

async fn start_round(mode: LldMode) -> LldResult<Vec<String>> {
    let mut containers = Vec::new();

    if !docker::check_image_exists(docker::IMAGE_DQLITE).await? {
        error!(
            "Docker image {:?} does not exist!\ndocker build -t {} -f ../docker/dqlite.Dockerfile ../",
            docker::IMAGE_DQLITE,
            docker::IMAGE_DQLITE
        );
        exit(1);
    }
    let id = docker::start_container(docker::IMAGE_DQLITE).await?;
    containers.push(id);

    if !docker::check_image_exists(docker::IMAGE_SERVER).await? {
        error!(
            "Docker image {:?} does not exist!\ndocker build -t {} -f ../docker/server.Dockerfile ../",
            docker::IMAGE_SERVER,
            docker::IMAGE_SERVER
        );
        exit(1);
    }
    let id = docker::start_container(docker::IMAGE_SERVER).await?;
    containers.push(id);

    Ok(containers)
}

async fn stop_round(containers: Vec<String>) -> LldResult<()> {
    for container_id in containers {
        docker::stop_container(&container_id).await?;
    }
    Ok(())
}

async fn start_step(
    environment: &Environment,
    mode: LldMode,
    count: usize,
    repeat: usize,
    duration: usize,
) -> LldResult<()> {
    let name = mode.to_string();
    for round in 0..repeat {
        eprintln!(
            "Start round {} of {}: {}, {} clients, {} ms",
            round + 1,
            repeat,
            name,
            count * 2,
            duration
        );

        let containers = start_round(mode).await?;

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
        .get_matches();

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

    let mut count = 1;

    println!("type,count,granted_avg,rejected_avg,timeout_avg,error_avg,granted_count,rejected_count,timeout_count,error_count");

    for _ in 1..max {
        for mode in [LldMode::Naive, LldMode::NaiveCaching, LldMode::Batching] {
            start_step(&environment, mode, count, repeat, duration).await?;
        }

        count *= 2;
    }

    Ok(())
}
