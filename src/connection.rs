use bytes::{BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
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
        let mut buffer = [0u8; 1024];
        let bytes = self.stream.read(&mut buffer).await?;

        if bytes == 0 {
            return Ok(None);
        }

        // let message = String::from_utf8_lossy(&buffer[..bytes]).to_string();
        let buffer = BytesMut::from(&buffer[..bytes]);
        let frame = Frame::parse_frame(buffer)?;
        println!("Received frame: {:?}", frame);
        Ok(Some(frame))
    }

    pub async fn write(&mut self, response: &[u8]) -> std::io::Result<()> {
        // Arrays are encoded by encoding each entry. All other frame types are
        // considered literals. For now, mini-redis is not able to encode
        // recursive frame structures. See below for more details.
        self.stream.write_all(response).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
