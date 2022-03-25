use lld_common::{LldError, LldResult};
use log::debug;
use tokio::process::Command;

const FILE_NATIVE_DQLITE: &str = "docker-compose-native-dqlite.yml";
const FILE_NATIVE_SQLITE: &str = "docker-compose-native-sqlite.yml";
const FILE_SCONE_DQLITE: &str = "docker-compose-scone-dqlite.yml";
const FILE_SCONE_SQLITE: &str = "docker-compose-scone-sqlite.yml";

#[derive(Debug, Clone, Copy)]
pub enum DockerComposeFile {
    NativeDqlite,
    NativeSqlite,
    SconeDqlite,
    SconeSqlite,
}

impl DockerComposeFile {
    const fn filename(self) -> &'static str {
        match self {
            DockerComposeFile::NativeDqlite => FILE_NATIVE_DQLITE,
            DockerComposeFile::NativeSqlite => FILE_NATIVE_SQLITE,
            DockerComposeFile::SconeDqlite => FILE_SCONE_DQLITE,
            DockerComposeFile::SconeSqlite => FILE_SCONE_SQLITE,
        }
    }

    pub async fn up(self) -> LldResult<()> {
        let mut command = Command::new("docker-compose");
        command.arg("-f").arg(self.filename());
        command.arg("up").arg("-d");

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

        Ok(())
    }

    pub async fn down(self) -> LldResult<()> {
        let mut command = Command::new("docker-compose");
        command.arg("-f").arg(self.filename());
        command.arg("down");

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

        Ok(())
    }
}
