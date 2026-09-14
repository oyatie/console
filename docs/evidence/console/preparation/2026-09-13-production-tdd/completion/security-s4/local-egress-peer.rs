//! Local HTTP receiver only. It neither projects data nor decides admission.
use crate::native_fixture::TestResult;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex, oneshot},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WireRequest {
    pub headers: Vec<u8>,
    pub body: Vec<u8>,
}
pub struct Peer {
    pub origin: url::Url,
    pub requests: Arc<Mutex<Vec<WireRequest>>>,
    shutdown: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<TestResult>,
}
impl Peer {
    pub async fn start() -> TestResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let origin = url::Url::parse(&format!("http://{}", listener.local_addr()?))?;
        let requests = Arc::new(Mutex::new(Vec::new()));
        let saved = requests.clone();
        let (shutdown, mut stop) = oneshot::channel();
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {biased;accepted=listener.accept()=>{let(mut stream,_)=accepted?;let mut header=Vec::new();loop{let byte=stream.read_u8().await?;header.push(byte);assert!(header.len()<=16*1024);if header.ends_with(b"\r\n\r\n"){break;}}
                let text=std::str::from_utf8(&header)?;assert!(!text.to_ascii_lowercase().contains("transfer-encoding:"),"fixed projected bytes must carry bounded Content-Length");let size=text.lines().find_map(|l|l.split_once(':').filter(|(k,_)|k.eq_ignore_ascii_case("content-length")).map(|(_,v)|v.trim().parse::<usize>())).transpose()?.unwrap_or(0);assert!(size<=8*1024*1024);let mut body=vec![0;size];stream.read_exact(&mut body).await?;saved.lock().await.push(WireRequest{headers:header,body});stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await?;},_=&mut stop=>break}
            }
            Ok(())
        });
        Ok(Self {
            origin,
            requests,
            shutdown,
            task,
        })
    }
    pub async fn finish(self) -> TestResult<Vec<WireRequest>> {
        let _ = self.shutdown.send(());
        self.task.await??;
        Ok(self.requests.lock().await.clone())
    }
}
