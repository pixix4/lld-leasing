use clap::Parser;
use std::{
    process::Command,
    time::{Duration, Instant},
};

use lld_leasing::{
    utils::{generate_random_id, http_request_leasing},
    LldResult,
};
use tokio::{spawn, task::JoinHandle, time::sleep};

async fn request() -> LldResult<bool> {
    let instance_id = generate_random_id::<64>();
    let application_id = generate_random_id::<1>();
    let result = http_request_leasing(&instance_id, &application_id).await?;

    Ok(result.is_some())
}

async fn start_concurrent_connections_round(count: usize) -> LldResult<(f64, usize)> {
    let h: Vec<JoinHandle<(Duration, bool)>> = (0..count)
        .map(|_| {
            spawn(async {
                let start = Instant::now();
                let result = request().await.unwrap();
                let duration = start.elapsed();
                (duration, result)
            })
        })
        .collect();

    let mut sum = 0;
    let count = h.len();
    let mut positive = 0;
    for handler in h {
        let (duration, result) = handler.await?;
        sum += duration.as_millis();
        if result {
            positive += 1;
        }
    }

    let avg = (sum as f64) / (count as f64);
    Ok((avg, positive))
}

#[derive(Parser)]
#[clap(
    version = "1.0",
    author = "Lars Westermann <lars.westermann@tu-dresden.de>"
)]
struct Opts {
    #[clap(short, long, default_value = "4")]
    repeat: usize,
    #[clap(short, long, default_value = "14")]
    max: usize,
    #[clap(default_value = "lld_leasing")]
    server_path: String,
}

async fn start_step(count: usize, repeat: usize, server_path: &str) -> LldResult<f64> {
    let mut sum = 0f64;
    for _ in 0..repeat {
        let mut child = Command::new(server_path).spawn()?;
        sleep(Duration::from_millis(200)).await;

        let result = start_concurrent_connections_round(count).await;

        child.kill()?;
        sleep(Duration::from_millis(200)).await;

        match result {
            Ok((avg, positive)) => {
                println!("   {:.4} with positive {}", avg, positive);
                sum += avg;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    let avg = sum / (repeat as f64);
    Ok(avg)
}

#[tokio::main]
async fn main() -> LldResult<()> {
    let opts: Opts = Opts::parse();

    let mut count = 1;

    for _ in 1..opts.max {
        println!(
            "Run bench for {} connections with {} repetitions",
            count, opts.repeat
        );
        let avg = start_step(count, opts.repeat, &opts.server_path).await?;
        println!("-> {:.4}", avg);
        count *= 2;
    }

    Ok(())
}
