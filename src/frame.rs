//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.

use bytes::{Bytes};

/// A frame in the Redis protocol.
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
    pub(crate) fn array() -> Frame {
        Frame::Array(vec![])
    }
}