#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn unit_connection_read_write() -> std::io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        // servidor (acepta 1 conexión y usa `Connection`)
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut conn = Connection::new(socket);
            let msg = timeout(Duration::from_secs(2), conn.read_message())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(msg.trim_end().starts_with("PING"));
            conn.write_response(b"+OK\r\n").await.unwrap();
        });

        // cliente
        let mut client = TcpStream::connect(addr).await?;
        client.write_all(b"PING\r\n").await?;
        let mut buf = vec![0u8; 16];
        let n = timeout(Duration::from_secs(2), client.read(&mut buf))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&buf[..n], b"+OK\r\n");

        server.await.unwrap();
        Ok(())
    }
}