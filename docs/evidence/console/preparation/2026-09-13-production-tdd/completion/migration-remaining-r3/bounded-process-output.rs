//! Bounded actual child receipt reader. stderr is discarded, never credential evidence.
use std::{process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Command};
pub async fn receipt_output(
    command: &mut Command,
    deadline: Duration,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn()?;
    let result = tokio::time::timeout(deadline, async {
        let mut bytes = Vec::new();
        child
            .stdout
            .take()
            .ok_or("missing actual stdout")?
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .await?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("batch receipt output exceeds4MiB".into());
        }
        if !child.wait().await?.success() {
            return Err("ordinary batch command unsuccessful".into());
        }
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(bytes)
    })
    .await;
    match result {
        Ok(Ok(bytes)) => Ok(bytes),
        failure => {
            // Explicitly reap owned direct child on cap/timeout/read error; never signal peers.
            if child.try_wait()?.is_none() {
                child.kill().await?;
            }
            child.wait().await?;
            match failure {
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e.into()),
                _ => unreachable!(),
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn actual_small_output_is_retained() {
        let mut c = Command::new("/usr/bin/printf");
        c.arg("receipt");
        assert_eq!(
            receipt_output(&mut c, Duration::from_secs(5))
                .await
                .unwrap(),
            b"receipt"
        );
    }
    #[tokio::test]
    async fn actual_unbounded_output_is_refused() {
        let mut c = Command::new("/usr/bin/yes");
        assert!(
            receipt_output(&mut c, Duration::from_secs(5))
                .await
                .unwrap_err()
                .to_string()
                .contains("exceeds4MiB")
        );
    }
    #[tokio::test]
    async fn actual_stalled_child_is_terminated() {
        let mut c = Command::new("/bin/sleep");
        c.arg("30");
        assert!(
            receipt_output(&mut c, Duration::from_millis(20))
                .await
                .is_err()
        );
    }
}
