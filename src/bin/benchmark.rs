#[macro_use]
extern crate log;

use clap::Parser;
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use lld_leasing::{
    utils::{
        generate_random_id, generate_random_u64, generate_random_u8, http_request_leasing,
        tcp_request_leasing,
    },
    LldResult,
};
use tokio::{spawn, task::JoinHandle, time::sleep};

enum ResultType {
    Granted,
    Rejected,
    Error,
}

async fn request(tcp: bool) -> LldResult<bool> {
    let result = if tcp {
        let application_id = generate_random_u8() as u64;
        let instance_id = generate_random_u64();
        let duration = 5000;
        tcp_request_leasing(application_id, instance_id, duration).await?
    } else {
        let application_id = generate_random_id::<1>();
        let instance_id = generate_random_id::<64>();
        let duration = 5000;
        http_request_leasing(&application_id, &instance_id, duration).await?
    };
    Ok(result.is_some())
}

async fn start_concurrent_connections_round(
    tcp: bool,
    count: usize,
) -> LldResult<(f64, i32, i32, i32)> {
    let h: Vec<JoinHandle<(Duration, ResultType)>> = (0..count)
        .map(|_| {
            spawn(async move {
                let start = Instant::now();
                let result_type = match request(tcp).await {
                    Ok(result) => {
                        if result {
                            ResultType::Granted
                        } else {
                            ResultType::Rejected
                        }
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        ResultType::Error
                    }
                };
                let duration = start.elapsed();
                (duration, result_type)
            })
        })
        .collect();

    let mut sum = 0;
    let count = h.len();
    let mut granted = 0;
    let mut rejected = 0;
    let mut errors = 0;
    for handler in h {
        let (duration, result) = handler.await?;
        sum += duration.as_millis();

        match result {
            ResultType::Granted => granted += 1,
            ResultType::Rejected => rejected += 1,
            ResultType::Error => errors += 1,
        }
    }

    let avg = (sum as f64) / (count as f64);
    Ok((avg, granted, rejected, errors))
}

async fn start_step(
    tcp: bool,
    batching: bool,
    count: usize,
    repeat: usize,
    server_path: &str,
) -> LldResult<()> {
    for _ in 0..repeat {
        let mut child = Command::new(server_path)
            .env("RUST_LOG", "ERROR")
            .env("USE_NAIVE", format!("{}", !batching))
            .stdout(Stdio::null())
            .spawn()?;
        sleep(Duration::from_millis(1000)).await;

        let result = start_concurrent_connections_round(tcp, count).await;

        child.kill()?;
        sleep(Duration::from_millis(200)).await;

        match result {
            Ok((avg, granted, rejected, errors)) => {
                println!(
                    "{},{},{},{},{},{}",
                    format!(
                        "{}-{}",
                        if tcp { "tcp" } else { "http" },
                        if batching { "batching" } else { "naive" }
                    ),
                    count,
                    avg,
                    granted,
                    rejected,
                    errors
                );
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

#[derive(Parser)]
#[clap(
    version = "1.0",
    author = "Lars Westermann <lars.westermann@tu-dresden.de>"
)]
struct Opts {
    #[clap(long, default_value = "4")]
    repeat: usize,
    #[clap(long, default_value = "8")]
    max: usize,
    #[clap(default_value = "lld_leasing")]
    server_path: String,
}

#[tokio::main]
async fn main() -> LldResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let opts: Opts = Opts::parse();

    let mut count = 1;

    println!("type,count,average,granted,rejected,errors");

    for _ in 1..opts.max {
        for batching in [false, true] {
            for tcp in [false, true] {
                start_step(tcp, batching, count, opts.repeat, &opts.server_path).await?;
            }
        }

        count *= 2;
    }

    Ok(())
}
