#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::io::{self, Write};
use std::{process::exit, time::Duration};

use clap::{App, Arg};
use reqwest::Client;
use tokio::{spawn, sync::mpsc, time::sleep};

use lld_common::{
    generate_random_id, generate_random_u64, get_current_time, http_request_client,
    http_request_leasing, tcp_request_leasing, Environment, LldResult,
};

enum RequestId {
    Http {
        application_id: String,
        instance_id: String,
        client: Client,
    },
    Tcp {
        application_id: u64,
        instance_id: u64,
    },
}
impl RequestId {
    fn get_application_id(&self) -> String {
        match self {
            RequestId::Http { application_id, .. } => format!("{}", application_id),
            RequestId::Tcp { application_id, .. } => format!("{}", application_id),
        }
    }

    fn get_instance_id(&self) -> String {
        match self {
            RequestId::Http { instance_id, .. } => format!("{}", instance_id),
            RequestId::Tcp { instance_id, .. } => format!("{}", instance_id),
        }
    }
}

async fn run_background_task(mut rx: mpsc::Receiver<u64>) -> LldResult<()> {
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

async fn request_leasing(
    environment: &Environment,
    request: &RequestId,
    duration: u64,
) -> LldResult<Option<u64>> {
    match request {
        RequestId::Http {
            application_id,
            instance_id,
            client,
        } => {
            http_request_leasing(client, environment, &application_id, &instance_id, duration).await
        }
        RequestId::Tcp {
            application_id,
            instance_id,
        } => tcp_request_leasing(environment, *application_id, *instance_id, duration).await,
    }
}

async fn run_single_leasing_client(
    environment: &Environment,
    request: &RequestId,
    duration: u64,
) -> LldResult<u64> {
    match request_leasing(environment, request, duration).await? {
        Some(validity) => Ok(validity),
        None => {
            error!("Could not get leasing, aborting!");
            exit(1);
        }
    }
}

async fn run_leasing_client_task(
    environment: &Environment,
    request: &RequestId,
    duration: u64,
    threshold: u64,
    tx: mpsc::Sender<u64>,
    init_validity: u64,
) -> LldResult<i32> {
    let now = get_current_time();
    let runtime = (init_validity - now) * threshold / 100;
    sleep(Duration::from_millis(runtime as u64)).await;

    loop {
        match request_leasing(environment, request, duration).await {
            Ok(Some(validity)) => {
                let now = get_current_time();

                tx.send(validity).await?;
                let runtime = (validity - now) * threshold / 100;
                sleep(Duration::from_millis(runtime as u64)).await;
            }
            Ok(None) => {
                error!("Could not get leasing, aborting!");
                exit(1);
            }
            Err(_) => {
                error!("Could not connect to leasing server, aborting!");
                exit(1);
            }
        }
    }
}

#[tokio::main]
async fn main() {
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
        .arg(Arg::with_name("id").env("LLD_APPLICATION_ID"))
        .arg(
            Arg::with_name("duration")
                .short("d")
                .long("duration")
                .env("LLD_DURATION"),
        )
        .arg(
            Arg::with_name("threshold")
                .short("t")
                .long("threshold")
                .env("LLD_THRESHOLD"),
        )
        .arg(Arg::with_name("tcp").long("tcp"))
        .get_matches();

    let http_uri = m
        .value_of("http_uri")
        .unwrap_or("https://mac.local:3030/request");
    let tcp_uri = m.value_of("tcp_uri").unwrap_or("127.0.0.1:3040");

    let use_tcp = m.is_present("tcp");

    let environment = Environment {
        http_request_uri: http_uri.to_owned(),
        tcp_request_uri: tcp_uri.to_owned(),
    };

    let application_id = m.value_of("id").unwrap_or_default();
    let duration = value_t!(m, "duration", u64).unwrap_or(5000);
    let threshold = value_t!(m, "threshold", u64).unwrap_or(50);

    let request = if use_tcp {
        RequestId::Tcp {
            application_id: application_id.parse().unwrap(),
            instance_id: generate_random_u64(),
        }
    } else {
        RequestId::Http {
            application_id: application_id.to_owned(),
            instance_id: generate_random_id::<64>(),
            client: http_request_client().unwrap(),
        }
    };

    let (tx, rx) = mpsc::channel::<u64>(8);

    info!("Configuration:");
    info!("    application_id: '{}'", request.get_application_id());
    info!("    instance_id: '{}'", request.get_instance_id());
    info!("    duration: '{}'", duration);
    info!("    threshold: '{}'", threshold);
    info!("");

    let init_validity;
    match run_single_leasing_client(&environment, &request, duration).await {
        Ok(validity) => {
            init_validity = validity;
        }
        Err(e) => {
            error!("{:?}", e);
            exit(1);
        }
    }

    spawn(async move {
        match run_background_task(rx).await {
            Ok(_) => {
                info!("Background task finished");
                exit(0);
            }
            Err(e) => {
                error!("{:?}", e);
                exit(1);
            }
        }
    });

    if tx.send(init_validity).await.is_err() {
        error!("Could not initiate background task!");
        exit(1)
    }
    match run_leasing_client_task(
        &environment,
        &request,
        duration,
        threshold,
        tx,
        init_validity,
    )
    .await
    {
        Ok(code) => {
            info!("Leasing task finished!");
            exit(code);
        }
        Err(e) => {
            error!("{:?}", e);
            exit(1);
        }
    }
}
