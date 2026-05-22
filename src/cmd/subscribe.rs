use bytes::Bytes;
use std::pin::Pin;
use tokio_stream::{Stream};

use crate::parse::{self, Parse};

/// Subscribes the client to one or more channels.
///
/// Once the client enters the subscribed state, it is not supposed to issue any
/// other commands, except for additional SUBSCRIBE, PSUBSCRIBE, UNSUBSCRIBE,
/// PUNSUBSCRIBE, PING and QUIT commands.
#[derive(Debug)]
pub struct Subscribe {
    channels: Vec<String>,
}

/// Unsubscribes the client from one or more channels.
///
/// When no channels are specified, the client is unsubscribed from all the
/// previously subscribed channels.
#[derive(Clone, Debug)]
pub struct Unsubscribe {
    channels: Vec<String>,
}

/// Stream of messages. The stream receives messages from the
/// `broadcast::Receiver`. We use `stream!` to create a `Stream` that consumes
/// messages. Because `stream!` values cannot be named, we box the stream using
/// a trait object.
type Messages = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

impl Subscribe {
    /// Creates a new `Subscribe` command to listen on the specified channels.
    pub(crate) fn new(channels: Vec<String>) -> Subscribe {
        Subscribe { channels }
    }
    
    pub fn parse_frames(parse: &mut Parse) -> std::result::Result<Subscribe, parse::ParseError> {
        println!("Parsing SUBSCRIBE command with parse: {:?}", parse);
        let channels = Vec::new();
        // while let Ok(channel) = parse.next_string() {
        //     channels.push(channel);
        // }
        Ok(Subscribe::new(channels))
    }
}