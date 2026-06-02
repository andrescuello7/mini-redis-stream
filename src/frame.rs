//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.

use std::io::Cursor;

use bytes::{Buf, Bytes, BytesMut};

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

impl Frame {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Frame::Simple(s) => format!("+{}\r\n", s).into_bytes(),
            Frame::Error(s) => format!("-{}\r\n", s).into_bytes(),
            Frame::Integer(n) => format!(":{}\r\n", n).into_bytes(),
            Frame::Bulk(data) => {
                let mut out = format!("${}\r\n", data.len()).into_bytes();
                out.extend_from_slice(data);
                out.extend_from_slice(b"\r\n");
                out
            }
            Frame::Null => b"$-1\r\n".to_vec(),
            Frame::Array(frames) => {
                let mut out = format!("*{}\r\n", frames.len()).into_bytes();
                for frame in frames {
                    out.extend(frame.encode());
                }
                out
            }
        }
    }

    pub(crate) fn parse_frame(mut buffer: BytesMut) -> std::io::Result<Frame> {
        let mut buf = Cursor::new(&buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;

                buf.set_position(0);

                let frame = Frame::parse(&mut buf)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("failed to parse frame: {e:?}"),
                        )
                    })?;

                buffer.advance(len);

                Ok(frame)
            }
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to parse frame: {e:?}"),
            )),
        }
    }

    /// Checks if an entire message can be decoded from `src`
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src) {
            Ok(b'+') => {
                get_line(src)?;
                Ok(())
            }
            Ok(b'-') => {
                get_line(src)?;
                Ok(())
            }
            Ok(b':') => {
                let _ = get_decimal(src)?;
                Ok(())
            }
            Ok(b'$') => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err(Error::Other(
                            format!("unsupported frame type: {:?}", line).into(),
                        ))
                    }
                    Ok(())
                } else {
                    // Read the bulk string
                    let len = usize::try_from(get_decimal(src)?)
                        .map_err(|_| Error::Other("invalid length".into()))?;
                    
                    // skip that number of bytes + 2 (\r\n).
                    skip(src, len + 2)
                }
            },
            Ok(b'*') => {
                let len = get_decimal(src)?;

                for _ in 0..len {
                    Frame::check(src)?;
                }

                Ok(())
            }
            _ => unimplemented!(),
        }
    }


    /// The message has already been validated with `check`.
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            b'+' => {
                // Read the line and convert it to `Vec<u8>`
                let line = get_line(src)?.to_vec();
                
                // Convert the line to String
                let string = String::from_utf8(line)
                    .map_err(|_| Error::Other("invalid utf8".into()))?;

                Ok(Frame::Simple(string))
            }
            b'-' => {
                // Read the line and convert it to Vec<u8>
                let line = get_line(src)?.to_vec();

                // Convert the line to String
                let string = String::from_utf8(line)
                    .map_err(|_| Error::Other("invalid utf8".into()))?;

                Ok(Frame::Error(string))
            }
            b':' => {
                let len = get_decimal(src)?;
                Ok(Frame::Integer(len))
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err(Error::Other(
                            "protocol error; invalid frame format".into()
                        ));
                    }

                    Ok(Frame::Null)
                } else {
                    // Read the bulk string
                    let len = usize::try_from(get_decimal(src)?)
                        .map_err(|_| Error::Other("invalid length".into()))?;

                    let n = len + 2;

                    if src.remaining() < n {
                        return Err(Error::Incomplete);
                    }

                    let data = Bytes::copy_from_slice(&src.chunk()[..len]);

                    // skip that number of bytes + 2 (\r\n).
                    skip(src, n)?;

                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                let len = usize::try_from(get_decimal(src)?)
                        .map_err(|_| Error::Other("invalid length".into()))?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(out))
            }
            _ => unimplemented!(),
        }
    }

}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

/// Read a new-line terminated decimal
fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(src)?;
    atoi::<u64>(line)
        .ok_or_else(|| Error::Other("protocol error; invalid frame format".into()))
}

/// Find a line
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}