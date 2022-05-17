use lld_common::{LldError, LldResult};
use log::debug;
use tokio::process::Command;

const FILE_NATIVE_SQLITE_NAIVE: &str = "compose/docker-compose-native-sqlite-naive.yml";
const FILE_NATIVE_SQLITE_CACHING: &str = "compose/docker-compose-native-sqlite-caching.yml";
const FILE_NATIVE_SQLITE_BATCHING: &str = "compose/docker-compose-native-sqlite-batching.yml";
const FILE_NATIVE_SQLITE_OPTIMIZED: &str = "compose/docker-compose-native-sqlite-optimized.yml";
const FILE_NATIVE_DQLITE: &str = "compose/docker-compose-native-dqlite.yml";
const FILE_NATIVE_DQLITE_NAIVE: &str = "compose/docker-compose-native-dqlite-naive.yml";
const FILE_SCONE_DQLITE: &str = "compose/docker-compose-scone-dqlite.yml";

#[derive(Debug, Clone, Copy)]
pub enum DockerComposeFile {
    NativeSqliteNaive,
    NativeSqliteCaching,
    NativeSqliteBatching,
    NativeDqliteNaive,
    NativeDqlite,
    NativeSqliteOptimized,
    SconeDqlite,
}

impl DockerComposeFile {
    const fn filename(self) -> &'static str {
        match self {
            DockerComposeFile::NativeSqliteNaive => FILE_NATIVE_SQLITE_NAIVE,
            DockerComposeFile::NativeSqliteCaching => FILE_NATIVE_SQLITE_CACHING,
            DockerComposeFile::NativeSqliteBatching => FILE_NATIVE_SQLITE_BATCHING,
            DockerComposeFile::NativeSqliteOptimized => FILE_NATIVE_SQLITE_OPTIMIZED,
            DockerComposeFile::NativeDqliteNaive => FILE_NATIVE_DQLITE_NAIVE,
            DockerComposeFile::NativeDqlite => FILE_NATIVE_DQLITE,
            DockerComposeFile::SconeDqlite => FILE_SCONE_DQLITE,
        }
    }

    pub async fn up(self) -> LldResult<()> {
        let mut command = Command::new("docker");
        command.arg("compose").arg("-f").arg(self.filename());
        command
            .arg("up")
            .arg("-d")
            .arg("--force-recreate")
            .arg("-V");

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
        let mut command = Command::new("docker");
        command.arg("compose").arg("-f").arg(self.filename());
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
