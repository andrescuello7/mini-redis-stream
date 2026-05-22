use crate::parse::{self, Parse};

#[derive(Debug)]
pub struct Get {
    /// Name of the key to get
    key: String,
}

impl Get {
    /// Create a new `Get` command which fetches `key`.
    pub fn new(key: impl ToString) -> Get {
        Get {
            key: key.to_string(),
        }
    }

    /// Get the key
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn parse_frames(parse: &mut Parse) -> std::result::Result<Get, parse::ParseError> {
        // let key = parse.next_string()?;
        println!("Parsing GET command with parse: {:?}", parse);
        Ok(Get::new("key"))
    }
}
