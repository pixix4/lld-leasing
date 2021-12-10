use std::ops::Add;
use std::time::{Duration, Instant};

use lld_common::{get_current_time, tcp_request_leasing, Environment, LldResult};
use tokio::time::timeout;
use tokio::{spawn, task::JoinHandle};

async fn request(environment: &Environment, application_id: u64, instance_id: u64) -> LoopResult {
    let duration = 5000;
    let instant = Instant::now();
    let result = timeout(
        Duration::from_secs(1),
        tcp_request_leasing(environment, application_id, instance_id, duration),
    )
    .await;
    let time = instant.elapsed().as_millis();

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{:?}", e);
            return LoopResult::new_timeout(time);
        }
    };

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{:?}", e);
            return LoopResult::new_error(time);
        }
    };

    if result.is_some() {
        LoopResult::new_granted(time)
    } else {
        LoopResult::new_rejected(time)
    }
}

pub struct LoopResult {
    pub granted_time: u128,
    pub granted_count: i32,
    pub rejected_time: u128,
    pub rejected_count: i32,
    pub timeout_time: u128,
    pub timeout_count: i32,
    pub error_time: u128,
    pub error_count: i32,
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
    environment: &Environment,
    application_id: u64,
    instance_id: u64,
    stop_at: u64,
) -> LldResult<LoopResult> {
    let mut result = LoopResult::new();

    while get_current_time() < stop_at {
        result = result + request(environment, application_id, instance_id).await;
    }

    Ok(result)
}

pub async fn start_concurrent_connections_round(
    environment: &Environment,
    count: usize,
    stop_at: u64,
) -> LldResult<LoopResult> {
    let mut h1: Vec<JoinHandle<LoopResult>> = (0..count as u64)
        .map(|application_id| {
            let e = environment.clone();
            spawn(async move { loop_requests(&e, application_id, 0, stop_at).await.unwrap() })
        })
        .collect();

    let mut h2: Vec<JoinHandle<LoopResult>> = (0..count as u64)
        .map(|application_id| {
            let e = environment.clone();
            spawn(async move { loop_requests(&e, application_id, 1, stop_at).await.unwrap() })
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
