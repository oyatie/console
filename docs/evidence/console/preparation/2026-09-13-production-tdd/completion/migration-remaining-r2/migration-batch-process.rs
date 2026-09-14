//! Actual ordinary AppRole::Migrate one-batch CLI and Unix output backpressure.
//! No seed-index process, success responder, postcommit hook or artificial receipt.
use console_app::{AppConfig, account_migration as migration, serving_admission as admission};
use sha2::{Digest, Sha256};
use std::{io::Write, os::fd::OwnedFd, os::unix::net::UnixStream, path::PathBuf, process::Stdio};
use tokio::process::{Child, Command};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub struct BatchProcess {
    child: Child,
    _receiver: UnixStream,
    _input: tempfile::NamedTempFile,
    pub stdout_capacity: usize,
}
async fn command(
    config: &AppConfig,
    request: &migration::BatchRequest,
) -> TestResult<(Command, tempfile::NamedTempFile)> {
    let installed = admission::load_installed_release(config).await?;
    let descriptor = serde_json::to_value(installed.descriptor())?;
    let binary = PathBuf::from(std::env::var("CONSOLE_ACCOUNT_MIGRATION_BINARY")?);
    let hash = hex::encode(Sha256::digest(tokio::fs::read(&binary).await?));
    assert_eq!(
        Some(hash.as_str()),
        descriptor["binary_sha256"].as_str(),
        "actual installed migration executable identity"
    );
    let mut input = tempfile::NamedTempFile::new()?;
    input.write_all(&serde_json::to_vec(request)?)?;
    input.flush()?;
    // Ordinary process serialization of AppConfig carries the actual migration
    // LOGIN/deployment authority settings. It cannot grant from request contents.
    let mut env = config.process_environment()?;
    env.push(("CONSOLE_APP_ROLE".into(), "migrate".into()));
    env.push(("CONSOLE_MIGRATION_OPERATION".into(), "account-batch".into()));
    env.push((
        "CONSOLE_MIGRATION_REQUEST_PATH".into(),
        input.path().to_string_lossy().into_owned(),
    ));
    let mut child = Command::new(binary);
    child
        .env_clear()
        .envs(env)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    Ok((child, input))
}
impl BatchProcess {
    pub async fn launch_blocked_output(
        config: &AppConfig,
        request: &migration::BatchRequest,
    ) -> TestResult<Self> {
        let (mut command, input) = command(config, request).await?;
        let (receiver, writer) = UnixStream::pair()?;
        let socket = socket2::SockRef::from(&writer);
        socket.set_send_buffer_size(1024)?;
        let stdout_capacity = socket.send_buffer_size()?;
        assert!(
            stdout_capacity <= 16384,
            "bounded Unix output socket prerequisite unavailable"
        );
        command.stdout(Stdio::from(OwnedFd::from(writer)));
        let child = command.spawn()?;
        Ok(Self {
            child,
            _receiver: receiver,
            _input: input,
            stdout_capacity,
        })
    }
    pub fn still_running(&mut self) -> TestResult<bool> {
        Ok(self.child.try_wait()?.is_none())
    }
    pub async fn kill_after_committed_receipt(mut self) -> TestResult {
        assert!(
            self.child.try_wait()?.is_none(),
            "actual owned migration process must still be alive after database commit"
        );
        self.child.kill().await?;
        self.child.wait().await?;
        Ok(())
    }
    pub async fn run_and_read(
        config: &AppConfig,
        request: &migration::BatchRequest,
    ) -> TestResult<migration::BatchReceipt> {
        let (mut command, _input) = command(config, request).await?;
        let output = crate::bounded_process_output::receipt_output(
            &mut command,
            std::time::Duration::from_secs(15),
        )
        .await?;
        Ok(serde_json::from_slice(&output)?)
    }
}
