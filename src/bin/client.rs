use std::{
    io::{self, Write},
    process::exit,
    time::Duration,
};

use clap::Parser;
use hyper::{Body, Client, Method, Request};
use rand::{thread_rng, RngCore};
use tokio::{spawn, sync::mpsc, time::sleep};

use lld_leasing::{api::RestLeasingResponse, database::get_current_time, LldResult};

async fn request(instance_id: &str, application_id: &str) -> LldResult<Option<i64>> {
    let client = Client::new();

    let req = Request::builder()
        .method(Method::POST)
        .uri("http://localhost:3030/request")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            "{{\"instance_id\":\"{}\", \"application_id\":\"{}\"}}",
            instance_id, application_id
        )))?;

    let mut resp = client.request(req).await?;

    let bytes = hyper::body::to_bytes(resp.body_mut()).await?;
    let result = String::from_utf8(bytes.into_iter().collect())?;

    let response: RestLeasingResponse = serde_json::from_str(&result)?;

    Ok(match response {
        RestLeasingResponse::Success {
            instance_id: _,
            application_id: _,
            validity,
        } => Some(validity),
        RestLeasingResponse::Error {
            instance_id: _,
            application_id: _,
        } => None,
    })
}

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
    let mut validity = rx.recv().await.unwrap_or_else(|| get_current_time());

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
    match request(instance_id, application_id).await? {
        Some(validity) => {
            return Ok(validity);
        }
        None => {
            println!("Could not get leasing, aborting!");
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
        match request(instance_id, application_id).await? {
            Some(validity) => {
                let now = get_current_time();

                tx.send(validity).await?;
                let runtime = (validity - now) * threshold / 100;
                sleep(Duration::from_millis(runtime as u64)).await;
            }
            None => {
                println!("Could not get leasing, aborting!");
                exit(1);
            }
        }
    }
}

fn generate_random_id<const T: usize>() -> String {
    let mut buffer = [0u8; T];
    thread_rng().fill_bytes(&mut buffer);
    base64::encode(&buffer)
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
            Ok(_) => exit(0),
            Err(e) => {
                eprintln!("{:?}", e);
                exit(1);
            }
        }
    });

    if tx.send(init_validity).await.is_err() {
        exit(1)
    }
    match run_leasing_client_task(&instance_id, &opts.id, opts.threshold, tx, init_validity).await {
        Ok(code) => exit(code),
        Err(e) => {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}
