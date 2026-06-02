use crate::{db::Db, frame::Frame, parse::{self, Parse}};

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn new(key: impl ToString) -> Get {
        Get { key: key.to_string() }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn parse_frames(parse: &mut Parse) -> Result<Get, parse::ParseError> {
        let key = parse.next_string()?;
        Ok(Get::new(key))
    }

    pub fn apply(&self, db: &Db) -> Frame {
        match db.get(&self.key) {
            Some(value) => Frame::Bulk(value),
            None => Frame::Null,
        }
    }
}
