use bytes::Bytes;
use std::time::Duration;

use crate::{db::Db, frame::Frame, parse::{self, Parse}};

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
    expire: Option<Duration>,
}

impl Set {
    pub fn new(key: impl ToString, value: Bytes, expire: Option<Duration>) -> Set {
        Set { key: key.to_string(), value, expire }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Bytes {
        &self.value
    }

    pub fn expire(&self) -> Option<Duration> {
        self.expire
    }

    pub fn parse_frames(parse: &mut Parse) -> Result<Set, parse::ParseError> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        // Optional EX <seconds> or PX <milliseconds>
        let expire = match parse.next_string() {
            Ok(opt) => match opt.to_uppercase().as_str() {
                "EX" => {
                    let secs = parse.next_string()?.parse::<u64>()
                        .map_err(|_| parse::ParseError::from("EX value must be an integer"))?;
                    Some(Duration::from_secs(secs))
                }
                "PX" => {
                    let ms = parse.next_string()?.parse::<u64>()
                        .map_err(|_| parse::ParseError::from("PX value must be an integer"))?;
                    Some(Duration::from_millis(ms))
                }
                _ => None,
            },
            Err(parse::ParseError::EndOfStream) => None,
            Err(e) => return Err(e),
        };

        Ok(Set::new(key, value, expire))
    }

    pub fn apply(&self, db: &Db) -> Frame {
        db.set(self.key.clone(), self.value.clone(), self.expire);
        Frame::Simple("OK".to_string())
    }
}
