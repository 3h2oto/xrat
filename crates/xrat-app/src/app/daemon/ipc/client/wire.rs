use std::path::Path;
#[cfg(unix)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(unix)]
pub(super) async fn send(endpoint: &Path, encoded: &[u8]) -> crate::app::Result<Vec<u8>> {
    let mut stream = tokio::net::UnixStream::connect(endpoint).await?;
    stream.write_all(encoded).await?;
    stream.write_all(b"\n").await?;
    stream.shutdown().await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    Ok(response)
}

#[cfg(windows)]
pub(super) async fn send(endpoint: &Path, encoded: &[u8]) -> crate::app::Result<Vec<u8>> {
    use tokio::io::AsyncWriteExt;
    use tokio::net::windows::named_pipe::ClientOptions;
    let exchange = async {
        let mut stream = loop {
            match ClientOptions::new().open(endpoint) {
                Ok(stream) => break stream,
                Err(error) if error.raw_os_error() == Some(231) => {
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
                Err(error) => return Err(error.into()),
            }
        };
        super::super::windows::verify_server(&stream)?;
        super::super::frame::write(&mut stream, encoded).await?;
        let response = super::super::frame::read(&mut stream).await?;
        stream.write_all(&[0]).await?;
        Ok(response)
    };
    tokio::time::timeout(std::time::Duration::from_secs(30), exchange)
        .await
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "daemon request timed out")
        })?
}
