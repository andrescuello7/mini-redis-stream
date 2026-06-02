use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::frame::Frame;

pub struct Connection {
    // The `TcpStream`. It is decorated with a `BufWriter`, which provides write
    // level buffering. The `BufWriter` implementation provided by Tokio is
    // sufficient for our needs.
    stream: BufWriter<TcpStream>,

    // The buffer for reading frames.
    buffer: BytesMut,
}

impl Connection {
    /// Create a new `Connection`, backed by `socket`. Read and write buffers
    /// are initialized.
    pub fn new(socket: TcpStream) -> Connection {
        println!("[+] New connection from {}", socket.peer_addr().unwrap());

        Connection {
            stream: BufWriter::new(socket),
            // The buffer is initialized with a capacity of 4 KB.
            // This means that the buffer can hold up to 4 KB of data before it needs to be resized.
            // The `BytesMut` type is a mutable byte buffer that can grow as needed.
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Option<Frame>> {
        let mut buffer: [u8; 1024] = [0u8; 1024];
        let bytes: usize = self.stream.read(&mut buffer).await?;

        if bytes == 0 {
            return Ok(None);
        }

        let buffer: BytesMut = BytesMut::from(&buffer[..bytes]);
        let frame: Frame = Frame::parse_frame(buffer)?;
        Ok(Some(frame))
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        self.stream.write_all(&frame.encode()).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
