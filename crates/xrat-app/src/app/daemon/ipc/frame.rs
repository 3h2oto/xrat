use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;

pub(super) async fn read(stream: &mut (impl AsyncRead + Unpin)) -> io::Result<Vec<u8>> {
    let size = stream.read_u32_le().await? as usize;
    if size == 0 || size > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid daemon message length",
        ));
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes).await?;
    Ok(bytes)
}

pub(super) async fn write(stream: &mut (impl AsyncWrite + Unpin), bytes: &[u8]) -> io::Result<()> {
    if bytes.is_empty() || bytes.len() > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid daemon message length",
        ));
    }
    stream.write_u32_le(bytes.len() as u32).await?;
    stream.write_all(bytes).await?;
    stream.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn framed_exchange_does_not_need_half_close() {
        let (mut client, mut server) = tokio::io::duplex(64);
        let worker = tokio::spawn(async move {
            assert_eq!(read(&mut server).await.unwrap(), b"request");
            write(&mut server, b"response").await.unwrap();
        });
        write(&mut client, b"request").await.unwrap();
        assert_eq!(read(&mut client).await.unwrap(), b"response");
        worker.await.unwrap();
    }

    #[tokio::test]
    async fn refuses_oversize_and_empty_messages_before_allocating() {
        for size in [0, MAX_MESSAGE_BYTES as u32 + 1, u32::MAX] {
            let mut source = &size.to_le_bytes()[..];
            assert_eq!(
                read(&mut source).await.unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
    }

    #[tokio::test]
    async fn rejects_truncated_frames() {
        let mut source = &[3, 0, 0, 0, 1][..];
        assert_eq!(
            read(&mut source).await.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
}
