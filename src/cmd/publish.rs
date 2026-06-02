use bytes::Bytes;

use crate::{db::Db, frame::Frame, parse::{self, Parse}};

#[derive(Debug)]
pub struct Publish {
    channel: String,
    message: Bytes,
}

impl Publish {
    pub(crate) fn new(channel: impl ToString, message: Bytes) -> Publish {
        Publish { channel: channel.to_string(), message }
    }

    pub fn parse_frames(parse: &mut Parse) -> Result<Publish, parse::ParseError> {
        let channel = parse.next_string()?;
        let message = parse.next_bytes()?;
        Ok(Publish::new(channel, message))
    }

    pub fn apply(&self, db: &Db) -> Frame {
        let count = db.publish(&self.channel, self.message.clone());
        Frame::Integer(count as u64)
    }
}
