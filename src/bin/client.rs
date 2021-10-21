use std::{
    io::{self, Write},
    process::exit,
    time::Duration,
};

use clap::Parser;
use hyper::{Body, Client, Method, Request};
use tokio::{spawn, sync::mpsc, time::sleep};

use lld_leasing::{api::RestLeasingResponse, database::get_current_time, LldResult};

async fn request(id: &str) -> LldResult<Option<i64>> {
    let client = Client::new();

    let req = Request::builder()
        .method(Method::POST)
        .uri("http://localhost:3030/request")
        .header("content-type", "application/json")
        .body(Body::from(format!("{{\"id\":\"{}\"}}", id)))?;

    let mut resp = client.request(req).await?;

    let bytes = hyper::body::to_bytes(resp.body_mut()).await?;
    let result = String::from_utf8(bytes.into_iter().collect())?;

    let response: RestLeasingResponse = serde_json::from_str(&result)?;

    Ok(match response {
        RestLeasingResponse::Success { id: _, validity } => Some(validity),
        RestLeasingResponse::Error { id: _ } => None,
    })
}

#[derive(Parser)]
#[clap(
    version = "1.0",
    author = "Lars Westermann <lars.westermann@tu-dresden.de>"
)]
struct Opts {
    id: String,
}

async fn run_background_task(mut rx: mpsc::Receiver<i64>) -> LldResult<()> {
    let mut validity = get_current_time();

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

async fn run_leasing_client_task(id: &str, tx: mpsc::Sender<i64>) -> LldResult<i32> {
    loop {
        match request(id).await? {
            Some(validity) => {
                let now = get_current_time();

                tx.send(validity).await?;
                let runtime = validity - now;
                sleep(Duration::from_millis(runtime as u64)).await;
            }
            None => {
                exit(1);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let (tx, rx) = mpsc::channel::<i64>(8);

    println!("Start leasing client with id '{}'", &opts.id);
    spawn(async move {
        match run_background_task(rx).await {
            Ok(_) => exit(0),
            Err(e) => {
                eprintln!("{:?}", e);
                exit(1);
            }
        }
    });

    match run_leasing_client_task(&opts.id, tx).await {
        Ok(code) => exit(code),
        Err(e) => {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}
