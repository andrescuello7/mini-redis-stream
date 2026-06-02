use crate::parse::{self, Parse, ParseError};

#[derive(Debug)]
pub struct Subscribe {
    channels: Vec<String>,
}

impl Subscribe {
    pub(crate) fn new(channels: Vec<String>) -> Subscribe {
        Subscribe { channels }
    }

    pub fn channels(&self) -> &[String] {
        &self.channels
    }

    pub fn parse_frames(parse: &mut Parse) -> Result<Subscribe, parse::ParseError> {
        let mut channels = Vec::new();
        loop {
            match parse.next_string() {
                Ok(ch) => channels.push(ch),
                Err(ParseError::EndOfStream) => break,
                Err(e) => return Err(e),
            }
        }
        if channels.is_empty() {
            return Err("SUBSCRIBE requires at least one channel".into());
        }
        Ok(Subscribe::new(channels))
    }
}
