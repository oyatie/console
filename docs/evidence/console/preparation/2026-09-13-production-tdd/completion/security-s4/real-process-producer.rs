//! Actual pinned executable, kernel-selected port0 and ordinary structured startup
//! event. Product prerequisite: log listener.local_addr(), PID and executable hash.
//! No fake responder, free-port probe, seed file or fixture-result process.
use crate::native_fixture::TestResult;
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use url::Url;
pub struct Process {
    pub child: Child,
    pub origin: Url,
    pub binary_sha256: String,
}
impl Process {
    pub async fn launch(
        binary: &Path,
        expected_sha256: &str,
        env: &[(String, String)],
    ) -> TestResult<Self> {
        let actual = hex::encode(Sha256::digest(tokio::fs::read(binary).await?));
        assert_eq!(actual, expected_sha256, "real executable pin");
        let mut command = Command::new(binary);
        command
            .env_clear()
            .envs(env.iter().cloned())
            .env("CONSOLE_HTTP_ADDR", "127.0.0.1:0")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true);
        let mut child = command.spawn()?;
        let pid = child.id().ok_or("actual process pid")?;
        let mut events = BufReader::new(child.stdout.take().ok_or("startup event stream")?);
        let bound = tokio::time::timeout(Duration::from_secs(30), async {
            let mut total = 0;
            loop {
                let mut line = String::new();
                let n = events.read_line(&mut line).await?;
                if n == 0 {
                    return Err("process closed before bound-listener event".into());
                }
                total += n;
                if total > 1024 * 1024 || n > 65536 {
                    return Err("bounded startup event stream exceeded".into());
                }
                let value: serde_json::Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if value["event"] != "http.listener.bound" {
                    continue;
                }
                assert_eq!(value["process_id"], pid);
                assert_eq!(value["binary_sha256"], actual);
                let address: std::net::SocketAddr = value["local_addr"]
                    .as_str()
                    .ok_or("actual listener address")?
                    .parse()?;
                assert!(address.ip().is_loopback() && address.port() != 0);
                return Ok::<_, Box<dyn std::error::Error + Send + Sync>>(Url::parse(&format!(
                    "http://{address}"
                ))?);
            }
        })
        .await??;
        // Drain normal logs so stdout backpressure cannot become test-created failure.
        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut events, &mut tokio::io::sink()).await;
        });
        Ok(Self {
            child,
            origin: bound,
            binary_sha256: actual,
        })
    }
    pub async fn ready(&mut self, client: &reqwest::Client) -> TestResult<reqwest::StatusCode> {
        self.ready_status(client, reqwest::StatusCode::OK).await
    }
    pub async fn ready_status(
        &mut self,
        client: &reqwest::Client,
        expected: reqwest::StatusCode,
    ) -> TestResult<reqwest::StatusCode> {
        let end = tokio::time::Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = self.child.try_wait()? {
                return Err(format!("actual application exited before readiness: {status}").into());
            }
            match client.get(self.origin.join("/readyz")?).send().await {
                Ok(response) if response.status() == expected => return Ok(response.status()),
                Ok(response) if tokio::time::Instant::now() >= end => {
                    return Err(format!(
                        "readiness deadline: expected {expected}, actual {}",
                        response.status()
                    )
                    .into());
                }
                Err(e) if tokio::time::Instant::now() >= end => return Err(e.into()),
                _ => tokio::time::sleep(Duration::from_millis(25)).await,
            }
        }
    }
    pub async fn stop(mut self) -> TestResult {
        self.child.kill().await?;
        self.child.wait().await?;
        Ok(())
    }
}
