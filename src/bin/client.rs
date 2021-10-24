use std::{
    io::{self, Write},
    process::exit,
    time::Duration,
};

use clap::Parser;
use tokio::{spawn, sync::mpsc, time::sleep};

use lld_leasing::{
    utils::{generate_random_id, get_current_time, http_request_leasing},
    LldResult,
};

#[derive(Parser)]
#[clap(
    version = "1.0",
    author = "Lars Westermann <lars.westermann@tu-dresden.de>"
)]
struct Opts {
    id: String,
    #[clap(default_value = "50")]
    threshold: i64,
}

async fn run_background_task(mut rx: mpsc::Receiver<i64>) -> LldResult<()> {
    let mut validity = rx.recv().await.unwrap_or_else(get_current_time);

    loop {
        if let Ok(v) = rx.try_recv() {
            validity = v;
        }

        let now = get_current_time();

        print!("\rThread is valid for {} ms    ", validity - now);
        io::stdout().flush()?;
        sleep(Duration::from_millis(50)).await;
    }
}

async fn run_single_leasing_client(instance_id: &str, application_id: &str) -> LldResult<i64> {
    match http_request_leasing(instance_id, application_id).await? {
        Some(validity) => Ok(validity),
        None => {
            eprintln!("Could not get leasing, aborting!");
            exit(1);
        }
    }
}

async fn run_leasing_client_task(
    instance_id: &str,
    application_id: &str,
    threshold: i64,
    tx: mpsc::Sender<i64>,
    init_validity: i64,
) -> LldResult<i32> {
    let now = get_current_time();
    let runtime = (init_validity - now) * threshold / 100;
    sleep(Duration::from_millis(runtime as u64)).await;

    loop {
        match http_request_leasing(instance_id, application_id).await? {
            Some(validity) => {
                let now = get_current_time();

                tx.send(validity).await?;
                let runtime = (validity - now) * threshold / 100;
                sleep(Duration::from_millis(runtime as u64)).await;
            }
            None => {
                eprintln!("Could not get leasing, aborting!");
                exit(1);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let instance_id = generate_random_id::<64>();

    let (tx, rx) = mpsc::channel::<i64>(8);

    println!("Configuration:");
    println!("    application_id: '{}'", &opts.id);
    println!("    instance_id: '{}'", &instance_id);
    println!("    threshold: '{}'", opts.threshold);
    println!();

    let init_validity;
    match run_single_leasing_client(&instance_id, &opts.id).await {
        Ok(validity) => {
            init_validity = validity;
        }
        Err(e) => {
            eprintln!("{:?}", e);
            exit(1);
        }
    }

    spawn(async move {
        match run_background_task(rx).await {
            Ok(_) => {
                println!("Background task finished");
                exit(0);
            }
            Err(e) => {
                eprintln!("{:?}", e);
                exit(1);
            }
        }
    });

    if tx.send(init_validity).await.is_err() {
        eprintln!("Could not initiate background task!");
        exit(1)
    }
    match run_leasing_client_task(&instance_id, &opts.id, opts.threshold, tx, init_validity).await {
        Ok(code) => {
            println!("Leasing task finished!");
            exit(code);
        }
        Err(e) => {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}
