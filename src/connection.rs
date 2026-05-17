use bytes::{Buf, BytesMut};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

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

    pub async fn read_frame(&mut self) -> std::io::Result<Option<String>> {
        // if 0 == self.stream.read_buf(&mut self.buffer).await? {
        //     // The remote closed the connection. For this to be a clean
        //     // shutdown, there should be no data in the read buffer. If
        //     // there is, this means that the peer closed the socket while
        //     // sending a frame.
        //     if self.buffer.is_empty() {
        //         return Ok(None);
        //     }
        // }
        let mut line = String::new();
        let mut reader = BufReader::new(&mut self.stream);
        let bytes = reader.read_line(&mut line).await?;

        println!("Received message: {}", line.trim());

        if bytes == 0 {
            return Ok(None);
        }
        Ok(Some(line))
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
