use bytes::Bytes;
use std::{vec};

use crate::frame::Frame;

/// Utility for parsing a command
///
/// Commands are represented as array frames. Each entry in the frame is a
/// "token". A `Parse` is initialized with the array frame and provides a
/// cursor-like API. Each command struct includes a `parse_frame` method that
/// uses a `Parse` to extract its fields.
#[derive(Debug)]
pub(crate) struct Parse {
    /// Array frame iterator.
    parts: vec::IntoIter<Frame>,
}

#[derive(Debug)]
pub(crate) enum ParseError {
    /// Attempting to extract a value failed due to the frame being fully
    /// consumed.
    EndOfStream,

    /// All other errors
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl Parse {
    /// Create a new `Parse` to parse the contents of `frame`.
    ///
    /// Returns `Err` if `frame` is not an array frame.
    pub(crate) fn new(frame: Frame) -> Result<Parse, ParseError> {
        let array = match frame {
            Frame::Array(array) => array,
            frame => return Err(format!("protocol error; expected array, got {frame:?}").into()),
        };

        Ok(Parse {
            parts: array.into_iter(),
        })
    }

    /// Return the next entry. Array frames are arrays of frames, so the next
    /// entry is a frame.
    fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(ParseError::EndOfStream)
    }

    /// Return the next entry as a string.
    ///
    /// If the next entry cannot be represented as a String, then an error is returned.
    pub(crate) fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            // Both `Simple` and `Bulk` representation may be strings. Strings
            // are parsed to UTF-8.
            //
            // While errors are stored as strings, they are considered separate
            // types.
            Frame::Simple(s) => Ok(s),
            Frame::Bulk(data) => str::from_utf8(&data[..])
                .map(|s| s.to_string())
                .map_err(|_| "protocol error; invalid string".into()),
            frame => Err(format!(
                "protocol error; expected simple frame or bulk frame, got {frame:?}"
            )
            .into()),
        }
    }

    pub fn to_string(&mut self) -> String {
        let mut result = String::new();
        for part in self.parts.by_ref() {
            match part {
                Frame::Simple(s) => result.push_str(&s),
                Frame::Error(e) => result.push_str(&e),
                Frame::Integer(i) => result.push_str(&i.to_string()),
                Frame::Bulk(b) => result.push_str(&String::from_utf8_lossy(&b)),
                Frame::Null => result.push_str("null"),
                Frame::Array(a) => {
                    for subpart in a {
                        match subpart {
                            Frame::Simple(s) => result.push_str(&s),
                            Frame::Error(e) => result.push_str(&e),
                            Frame::Integer(i) => result.push_str(&i.to_string()),
                            Frame::Bulk(b) => result.push_str(&String::from_utf8_lossy(&b)),
                            Frame::Null => result.push_str("null"),
                            _ => {}
                        }
                    }
                }
            }
        }
        result
    }
}


impl From<String> for ParseError {
    fn from(src: String) -> ParseError {
        ParseError::Other(src.into())
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        src.to_string().into()
    }
}