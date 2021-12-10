use lld_common::{LldError, LldResult};
use log::debug;
use tokio::process::Command;

pub const IMAGE_DQLITE: &str = "pixix4/lld-dqlite:latest";
pub const IMAGE_SERVER: &str = "pixix4/lld-server:latest";

pub async fn check_image_exists(image: &str) -> LldResult<bool> {
    let mut command = Command::new("docker");
    command.arg("images").arg("-q").arg(image);

    let output = command.output().await?;
    if !output.status.success() {
        return Err(LldError::WrappedError(
            "docker error",
            format!(
                "Command exited with status code {:?}!",
                output.status.code()
            ),
        ));
    }

    Ok(!output.stdout.is_empty())
}

pub async fn start_container(
    image: &str,
    ports: &[u16],
    env: &[(&str, &str)],
) -> LldResult<String> {
    let mut command = Command::new("docker");
    command.arg("run").arg("--rm").arg("-d");

    for port in ports {
        command.arg("-p").arg(format!("{}:{}", port, port));
    }

    for (key, value) in env {
        command.arg("--env").arg(format!("{}={}", key, value));
    }

    command.arg(image);

    debug!("Docker command: {:?}", command);
    let output = command.output().await?;
    debug!("Docker output: {:?}", output);
    if !output.status.success() {
        return Err(LldError::WrappedError(
            "docker error",
            format!(
                "Command exited with status code {:?}!",
                output.status.code()
            ),
        ));
    }

    let container_id = String::from_utf8(output.stdout)?.trim().to_owned();
    Ok(container_id)
}

pub async fn stop_container(container_id: &str) -> LldResult<bool> {
    let mut command = Command::new("docker");
    command.arg("stop").arg("-t").arg("0").arg(container_id);

    debug!("Docker command: {:?}", command);
    let output = command.output().await?;
    debug!("Docker output: {:?}", output);
    Ok(output.status.success())
}
