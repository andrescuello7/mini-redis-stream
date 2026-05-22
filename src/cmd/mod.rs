mod get;
use std::io::Error;

pub use get::Get;

mod set;
pub use set::Set;

mod publish;
pub use publish::Publish;

mod subscribe;
pub use subscribe::{Subscribe, Unsubscribe};

use crate::parse::{self, Parse};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    Publish(Publish),
    Subscribe(Subscribe),
    // Unsubscribe(Unsubscribe),
}

impl Command {
    /// Parse a command from a received frame.
    ///
    /// The `Frame` must represent a Redis command supported by `mini-redis` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: String) -> std::result::Result<Command, parse::ParseError> {
        // The frame value is decorated with `Parse`. `Parse` provides a
        // "cursor" like API which makes parsing the command easier.
        //
        // The frame value must be an array variant. Any other frame variants
        // result in an error being returned.
        let mut parse = Parse::new(frame);
        // println!("Received frame desde COMMAND: {:?}", parse);

        // All redis commands begin with the command name as a string. The name
        // is read and converted to lower cases in order to do case sensitive
        // matching.
        let binding = String::from("value");
        let mut parts = binding.split_whitespace();
        let command_name = parts.next().ok_or("empty command")?;
        
        let command: Command = match command_name.to_lowercase().as_str() {
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            "publish" => Command::Publish(Publish::parse_frames(&mut parse)?),
            "subscribe" => Command::Subscribe(Subscribe::parse_frames(&mut parse)?),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                // return Ok(Command::Unknown(Unknown::new(command_name)));
                return Err(format!("unknown command {command_name}").into());
            }
        };

        // The command has been successfully parsed
        Ok(command)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
