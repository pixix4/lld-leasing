use std::{os::unix::prelude::FromRawFd, process::Stdio};

use lld_common::{LldError, LldResult};
use log::{debug, warn};
use tokio::process::Command;

const IMAGE_DQLITE: &str = "pixix4/lld-dqlite:latest";
const IMAGE_NATIVE_DQLITE: &str = "pixix4/lld-native-dqlite:latest";
const IMAGE_NATIVE_SQLITE: &str = "pixix4/lld-native-sqlite:latest";
const IMAGE_SCONE_DQLITE: &str = "pixix4/lld-scone-dqlite:latest";
const IMAGE_SCONE_SQLITE: &str = "pixix4/lld-scone-sqlite:latest";

#[derive(Debug, Clone)]
pub struct ContainerRef(String);

impl ContainerRef {
    pub async fn stop_container(self) -> LldResult<bool> {
        stop_container(&self).await
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DockerImage {
    DqliteServer,
    LldNativeDqlite,
    LldNativeSqlite,
    LldSconeDqlite,
    LldSconeSqlite,
}

impl DockerImage {
    const fn image_tag(self) -> &'static str {
        match self {
            DockerImage::DqliteServer => IMAGE_DQLITE,
            DockerImage::LldNativeDqlite => IMAGE_NATIVE_DQLITE,
            DockerImage::LldNativeSqlite => IMAGE_NATIVE_SQLITE,
            DockerImage::LldSconeDqlite => IMAGE_SCONE_DQLITE,
            DockerImage::LldSconeSqlite => IMAGE_SCONE_SQLITE,
        }
    }

    pub async fn check_image_exsits(self) -> LldResult<bool> {
        match self {
            DockerImage::DqliteServer => check_image_exists(self.image_tag()).await,
            DockerImage::LldNativeDqlite => check_image_exists(self.image_tag()).await,
            DockerImage::LldNativeSqlite => check_image_exists(self.image_tag()).await,
            DockerImage::LldSconeDqlite => check_image_exists(self.image_tag()).await,
            DockerImage::LldSconeSqlite => check_image_exists(self.image_tag()).await,
        }
    }

    pub async fn build_image(self) -> LldResult<()> {
        warn!("Rebuilding docker image {:?}!", self.image_tag());
        match self {
            DockerImage::DqliteServer => {
                build_image("docker/dqlite.Dockerfile", self.image_tag()).await
            }
            DockerImage::LldNativeDqlite => {
                build_image("docker/server-native-dqlite.Dockerfile", self.image_tag()).await
            }
            DockerImage::LldNativeSqlite => {
                build_image("docker/server-native-sqlite.Dockerfile", self.image_tag()).await
            }
            DockerImage::LldSconeDqlite => {
                build_image("docker/server-scone-dqlite.Dockerfile", self.image_tag()).await
            }
            DockerImage::LldSconeSqlite => {
                build_image("docker/server-scone-sqlite.Dockerfile", self.image_tag()).await
            }
        }
    }

    pub async fn build_image_if_not_exists(self, force_build: bool) -> LldResult<()> {
        if force_build || !self.check_image_exsits().await? {
            self.build_image().await?;
        }

        Ok(())
    }

    pub async fn start_container(self, env: &[(&str, &str)]) -> LldResult<ContainerRef> {
        match self {
            DockerImage::DqliteServer => {
                start_container(self.image_tag(), &[24000, 25000, 26000], env).await
            }
            DockerImage::LldNativeDqlite => {
                start_container(self.image_tag(), &[3030, 3040], env).await
            }
            DockerImage::LldNativeSqlite => {
                start_container(self.image_tag(), &[3030, 3040], env).await
            }
            DockerImage::LldSconeDqlite => {
                start_container(self.image_tag(), &[3030, 3040], env).await
            }
            DockerImage::LldSconeSqlite => {
                start_container(self.image_tag(), &[3030, 3040], env).await
            }
        }
    }
}

async fn check_image_exists(image: &str) -> LldResult<bool> {
    let mut command = Command::new("docker");
    command.arg("images").arg("-q").arg(image);

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

    Ok(!output.stdout.is_empty())
}

async fn build_image(dockerfile: &str, image: &str) -> LldResult<()> {
    let mut command = Command::new("docker");
    command
        .arg("build")
        .arg("-t")
        .arg(image)
        .arg("-f")
        .arg(dockerfile)
        .arg(".");

    command.stdout(unsafe { Stdio::from_raw_fd(libc::dup(2)) });

    debug!("Docker command: {:?}", command);
    let exit_status = command.status().await?;
    debug!("Docker exit status: {:?}", exit_status);

    if !exit_status.success() {
        return Err(LldError::WrappedError(
            "docker error",
            format!("Command exited with status code {:?}!", exit_status.code()),
        ));
    }

    Ok(())
}

async fn start_container(
    image: &str,
    ports: &[u16],
    env: &[(&str, &str)],
) -> LldResult<ContainerRef> {
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
    Ok(ContainerRef(container_id))
}

async fn stop_container(container_id: &ContainerRef) -> LldResult<bool> {
    let mut command = Command::new("docker");
    command.arg("stop").arg("-t").arg("0").arg(&container_id.0);

    debug!("Docker command: {:?}", command);
    let output = command.output().await?;
    debug!("Docker output: {:?}", output);

    Ok(output.status.success())
}
