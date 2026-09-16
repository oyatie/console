//! Real configured existing S3 GET transport; local servers are wire peers, not
//! replacement storage implementations. No real provider, DELETE or TLS claim.
use console_platform_storage::{S3StorageConfig, SeaweedS3Storage};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn read_request(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 1024];
    loop {
        let size = stream.read(&mut buffer).await?;
        if size == 0 {
            return Err("HTTP peer closed before request".into());
        }
        bytes.extend_from_slice(&buffer[..size]);
        if bytes.windows(4).any(|v| v == b"\r\n\r\n") {
            break;
        }
        if bytes.len() > 8192 {
            return Err("test HTTP request exceeded bounded header".into());
        }
    }
    assert!(bytes.starts_with(b"GET "));
    Ok(bytes)
}
fn config(endpoint: String) -> S3StorageConfig {
    S3StorageConfig {
        endpoint_url: endpoint,
        region: "us-east-1".into(),
        access_key_id: "TEST_ONLY".into(),
        secret_access_key: "TEST_ONLY".into(),
        primary_bucket: "fixture".into(),
        replica_bucket: "fixture".into(),
        force_path_style: true,
    }
}

#[tokio::test]
async fn configured_storage_get_returns_exact_direct_endpoint_bytes() -> Result<()> {
    let server = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", server.local_addr()?);
    let task = tokio::spawn(async move {
        let (mut stream, _) = server.accept().await?;
        let request = read_request(&mut stream).await?;
        assert!(request.starts_with(b"GET /fixture/TEST_ONLY-direct "));
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nTEST_ONLY").await?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });
    let storage = SeaweedS3Storage::from_config(&config(endpoint)).await?;
    let actual = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        storage.get_bytes("fixture", "TEST_ONLY-direct"),
    )
    .await??;
    task.await??;
    assert_eq!(actual, (b"TEST_ONLY".to_vec(), Some("text/plain".into())));
    Ok(())
}

#[tokio::test]
async fn configured_storage_get_never_follows_redirect_to_second_endpoint() -> Result<()> {
    for status in [301, 302, 303, 307, 308] {
        let source = TcpListener::bind("127.0.0.1:0").await?;
        let destination = TcpListener::bind("127.0.0.1:0").await?;
        let source_url = format!("http://{}", source.local_addr()?);
        let destination_url = format!(
            "http://{}/TEST_ONLY-redirect-canary",
            destination.local_addr()?
        );
        let (shutdown, mut stopped) = oneshot::channel::<()>();
        let target = tokio::spawn(async move {
            let mut requests = 0usize;
            loop {
                tokio::select! {biased;
                    connection=destination.accept()=>{
                        let (mut stream,_)=connection?;read_request(&mut stream).await?;requests+=1;
                        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nTEST_ONLY").await?;
                    }
                    _=&mut stopped=>break,
                }
            }
            Ok::<usize, Box<dyn std::error::Error + Send + Sync>>(requests)
        });
        let first = tokio::spawn(async move {
            let (mut stream, _) = source.accept().await?;
            read_request(&mut stream).await?;
            let response = format!(
                "HTTP/1.1 {status} Redirect\r\nLocation: {destination_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
        let storage = SeaweedS3Storage::from_config(&config(source_url)).await?;
        let actual = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            storage.get_bytes("fixture", "original"),
        )
        .await;
        let _ = shutdown.send(());
        first.await??;
        let requests = target.await??;
        assert!(
            actual?.is_err(),
            "HTTP{status} redirect must be refused by actual configured transport"
        );
        assert_eq!(
            requests, 0,
            "actual client crossed endpoint on HTTP{status}"
        );
    }
    Ok(())
}
