//! Shared finite framing for two demonstrated disposable loopback wire tests.
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
const MAX_FRAME: usize = 1024 * 1024;
pub(crate) fn invalid_wire() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "finite fixture wire protocol failure",
    )
}

// Raw startup/auth/row bytes exist only in bounded relay buffers, never receipts/logs.
pub(crate) async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<(u8, Vec<u8>)> {
    let tag = reader.read_u8().await?;
    let size = reader.read_u32().await? as usize;
    if !(4..=MAX_FRAME).contains(&size) {
        return Err(invalid_wire());
    }
    let mut body = vec![0; size - 4];
    reader.read_exact(&mut body).await?;
    Ok((tag, body))
}
pub(crate) async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    tag: u8,
    body: &[u8],
) -> std::io::Result<()> {
    if body.len() > MAX_FRAME - 4 {
        return Err(invalid_wire());
    }
    writer.write_u8(tag).await?;
    writer.write_u32((body.len() + 4) as u32).await?;
    writer.write_all(body).await?;
    writer.flush().await
}
pub(crate) async fn read_startup<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Vec<u8>> {
    let len = reader.read_u32().await? as usize;
    if !(8..=MAX_FRAME).contains(&len) {
        return Err(invalid_wire());
    }
    let mut startup = vec![0; len - 4];
    reader.read_exact(&mut startup).await?;
    if startup[..4] != 196608u32.to_be_bytes() {
        return Err(invalid_wire());
    }
    Ok(startup)
}
