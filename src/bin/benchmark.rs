extern crate log;

use clap::Parser;
use log::error;
use std::{
    fs,
    ops::Add,
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use lld_leasing::{
    utils::{get_current_time, tcp_request_leasing},
    LldResult,
};
use tokio::{
    spawn,
    task::JoinHandle,
    time::{sleep, timeout},
};

async fn request(application_id: u64, instance_id: u64) -> LoopResult {
    let duration = 5000;
    let instant = Instant::now();
    let result = timeout(
        Duration::from_secs(1),
        tcp_request_leasing(application_id, instance_id, duration),
    )
    .await;
    let time = instant.elapsed().as_millis();

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            error!("{:?}", e);
            return LoopResult::new_timeout(time);
        }
    };
    sleep(Duration::from_millis(500)).await;

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            error!("{:?}", e);
            return LoopResult::new_error(time);
        }
    };

    if result.is_some() {
        LoopResult::new_granted(time)
    } else {
        LoopResult::new_rejected(time)
    }
}

struct LoopResult {
    granted_time: u128,
    granted_count: i32,
    rejected_time: u128,
    rejected_count: i32,
    timeout_time: u128,
    timeout_count: i32,
    error_time: u128,
    error_count: i32,
}

impl LoopResult {
    pub fn new() -> Self {
        Self {
            granted_time: 0,
            granted_count: 0,
            rejected_time: 0,
            rejected_count: 0,
            timeout_time: 0,
            timeout_count: 0,
            error_time: 0,
            error_count: 0,
        }
    }

    pub fn new_granted(time: u128) -> Self {
        Self {
            granted_time: time,
            granted_count: 1,
            rejected_time: 0,
            rejected_count: 0,
            timeout_time: 0,
            timeout_count: 0,
            error_time: 0,
            error_count: 0,
        }
    }

    pub fn new_rejected(time: u128) -> Self {
        Self {
            granted_time: 0,
            granted_count: 0,
            rejected_time: time,
            rejected_count: 1,
            timeout_time: 0,
            timeout_count: 0,
            error_time: 0,
            error_count: 0,
        }
    }

    pub fn new_timeout(time: u128) -> Self {
        Self {
            granted_time: 0,
            granted_count: 0,
            rejected_time: 0,
            rejected_count: 0,
            timeout_time: time,
            timeout_count: 1,
            error_time: 0,
            error_count: 0,
        }
    }

    pub fn new_error(time: u128) -> Self {
        Self {
            granted_time: 0,
            granted_count: 0,
            rejected_time: 0,
            rejected_count: 0,
            timeout_time: 0,
            timeout_count: 0,
            error_time: time,
            error_count: 1,
        }
    }
}

impl Add<LoopResult> for LoopResult {
    type Output = LoopResult;

    fn add(self, other: LoopResult) -> LoopResult {
        LoopResult {
            granted_time: self.granted_time + other.granted_time,
            granted_count: self.granted_count + other.granted_count,
            rejected_time: self.rejected_time + other.rejected_time,
            rejected_count: self.rejected_count + other.rejected_count,
            timeout_time: self.timeout_time + other.timeout_time,
            timeout_count: self.timeout_count + other.timeout_count,
            error_time: self.error_time + other.error_time,
            error_count: self.error_count + other.error_count,
        }
    }
}

async fn loop_requests(
    application_id: u64,
    instance_id: u64,
    stop_at: u64,
) -> LldResult<LoopResult> {
    let mut result = LoopResult::new();

    while get_current_time() < stop_at {
        result = result + request(application_id, instance_id).await;
    }

    Ok(result)
}

async fn start_concurrent_connections_round(count: usize, stop_at: u64) -> LldResult<LoopResult> {
    let mut h1: Vec<JoinHandle<LoopResult>> = (0..count as u64)
        .map(|application_id| {
            spawn(async move {
                //let application_id = generate_random_u8() as u64;
                loop_requests(application_id, 0, stop_at).await.unwrap()
            })
        })
        .collect();

    let mut h2: Vec<JoinHandle<LoopResult>> = (0..count as u64)
        .map(|application_id| {
            spawn(async move {
                //let application_id = generate_random_u8() as u64;
                loop_requests(application_id, 1, stop_at).await.unwrap()
            })
        })
        .collect();

    h1.append(&mut h2);

    let mut result = LoopResult::new();
    for handler in h1 {
        let r = handler.await?;
        result = result + r;
    }

    Ok(result)
}

async fn start_step(
    disable_batching: bool,
    disable_cache: bool,
    count: usize,
    repeat: usize,
    duration: usize,
    server_path: &str,
) -> LldResult<()> {
    let name = format!(
        "{}-{}",
        if disable_batching {
            "naive"
        } else {
            "batching"
        },
        if disable_cache { "nocache" } else { "cache" }
    );
    for round in 0..repeat {
        eprintln!(
            "Start round {} of {}: {}, {} clients, {} ms",
            round + 1,
            repeat,
            name,
            count * 2,
            duration
        );

        let db = Path::new("database.db");
        if db.exists() {
            fs::remove_file(db)?;
        }

        let dbs = dqlite_start().await?;

        let mut child = Command::new(server_path)
            .env("RUST_LOG", "ERROR")
            .env("DISABLE_BATCHING", format!("{}", disable_batching))
            .env("DISABLE_CACHE", format!("{}", disable_cache))
            .stdout(Stdio::null())
            .spawn()?;
        sleep(Duration::from_millis(1000)).await;

        let stop_at = get_current_time() + (duration as u64);
        let result = start_concurrent_connections_round(count, stop_at).await;

        child.kill()?;
        sleep(Duration::from_millis(200)).await;

        dqlite_stop(dbs).await?;

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

#[derive(Parser)]
#[clap(
    version = "1.0",
    author = "Lars Westermann <lars.westermann@tu-dresden.de>"
)]
struct Opts {
    #[clap(long, default_value = "1")]
    repeat: usize,
    #[clap(long, default_value = "3")]
    max: usize,
    #[clap(long, default_value = "3000")]
    duration: usize,
    #[clap(default_value = "lld_leasing")]
    server_path: String,
}

#[tokio::main]
async fn main() -> LldResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let opts: Opts = Opts::parse();

    let mut count = 1;

    println!("type,count,granted_avg,rejected_avg,timeout_avg,error_avg,granted_count,rejected_count,timeout_count,error_count");

    for _ in 1..opts.max {
        for disable_batching in [false, true] {
            for disable_cache in [false, true] {
                start_step(
                    disable_batching,
                    disable_cache,
                    count,
                    opts.repeat,
                    opts.duration,
                    &opts.server_path,
                )
                .await?;
            }
        }

        count *= 2;
    }

    Ok(())
}

async fn dqlite_start() -> LldResult<(Child, Child, Child)> {
    let db1 = Command::new("/root/server")
        .env("SERVER_ADDRESS", "127.0.0.1")
        .env("NODE_ID", "1")
        .env("PORT", "24000")
        .stdout(Stdio::null())
        .spawn()?;

    let db2 = Command::new("/root/server")
        .env("SERVER_ADDRESS", "127.0.0.1")
        .env("NODE_ID", "2")
        .env("PORT", "25000")
        .stdout(Stdio::null())
        .spawn()?;

    let db3 = Command::new("/root/server")
        .env("SERVER_ADDRESS", "127.0.0.1")
        .env("NODE_ID", "3")
        .env("PORT", "26000")
        .stdout(Stdio::null())
        .spawn()?;

    sleep(Duration::from_millis(1000)).await;

    Ok((db1, db2, db3))
}

async fn dqlite_stop(mut childs: (Child, Child, Child)) -> LldResult<()> {
    childs.0.kill()?;
    childs.1.kill()?;
    childs.2.kill()?;
    sleep(Duration::from_millis(200)).await;

    Command::new("bash")
        .arg("-c")
        .arg("rm -rf /tmp/dqlite-rs*")
        .stdout(Stdio::null())
        .spawn()?
        .wait()?;

    Ok(())
}
