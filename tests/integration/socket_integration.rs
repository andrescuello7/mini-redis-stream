use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn integration_connection_flow() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // servidor simple que responde +OK a dos mensajes
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        // IMPORTANTE: aquí usamos la misma lógica que el servidor de la app.
        let mut reader = tokio::io::BufReader::new(socket);
        let mut line = String::new();
        for _ in 0..2 {
            let n = tokio::time::timeout(Duration::from_secs(2), reader.read_line(&mut line))
                .await
                .unwrap()
                .unwrap();
            if n == 0 { break; }
            line.clear();
            let mut w = reader.get_mut();
            w.write_all(b"+OK\r\n").await.unwrap();
            w.flush().await.unwrap();
        }
    });

    // cliente: envía dos comandos y verifica dos respuestas
    let mut client = TcpStream::connect(addr).await?;
    client.write_all(b"ECHO hello\r\n").await?;
    let mut buf = [0u8; 16];
    let n1 = timeout(Duration::from_secs(2), client.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&buf[..n1], b"+OK\r\n");

    client.write_all(b"PING\r\n").await?;
    let n2 = timeout(Duration::from_secs(2), client.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&buf[..n2], b"+OK\r\n");

    server.await.unwrap();
    Ok(())
}