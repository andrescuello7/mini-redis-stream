use bytes::{BytesMut};
use tokio::io::{BufWriter};
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
        Connection {
            stream: BufWriter::new(socket),
            // The buffer is initialized with a capacity of 4 KB. 
            // This means that the buffer can hold up to 4 KB of data before it needs to be resized. 
            // The `BytesMut` type is a mutable byte buffer that can grow as needed.
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }
}