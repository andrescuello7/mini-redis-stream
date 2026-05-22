use bytes::Bytes;

use crate::parse::{self, Parse};

/// Posts a message to the given channel.
///
/// Send a message into a channel without any knowledge of individual consumers.
/// Consumers may subscribe to channels in order to receive the messages.
///
/// Channel names have no relation to the key-value namespace. Publishing on a
/// channel named "foo" has no relation to setting the "foo" key.
#[derive(Debug)]
pub struct Publish {
    /// Name of the channel on which the message should be published.
    channel: String,

    /// The message to publish.
    message: Bytes,
}

impl Publish {
    /// Create a new `Publish` command which sends `message` on `channel`.
    pub(crate) fn new(channel: impl ToString, message: Bytes) -> Publish {
        Publish {
            channel: channel.to_string(),
            message,
        }
    }

    pub fn parse_frames(parse: &mut Parse) -> std::result::Result<Publish, parse::ParseError> {
        println!("Parsing PUBLISH command with parse: {:?}", parse);
        Ok(Publish::new("channel", "message".into()))
    }
}
