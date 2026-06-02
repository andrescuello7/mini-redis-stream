use bytes::Bytes;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, Semaphore};
use std::sync::Arc;

use crate::cmd::{Command, Subscribe};
use crate::connection::Connection;
use crate::db::{Db, DbDropGuard};
use crate::frame::Frame;

pub struct Listener {
    db_holder: DbDropGuard,
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

const STRING_CONNECTION: &str = "127.0.0.1:6379";
const MAX_CONNECTIONS: usize = 250;

struct Handler {
    db: Db,
    connection: Connection,
}

pub async fn run() -> std::io::Result<Listener> {
    let listener: Listener = Listener {
        db_holder: DbDropGuard::new(),
        listener: TcpListener::bind(STRING_CONNECTION).await?,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    let mut server: Listener = listener;

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                println!("failed to accept connections: {}", err);
            }
        },
    }
    println!("Listening on {}", server.listener.local_addr()?);
    Ok(Listener { listener: server.listener, limit_connections: server.limit_connections, db_holder: server.db_holder })
}

impl Listener {
    pub async fn run(&mut self) -> Result<Vec<u8>> {
        println!("accepting inbound connections");

        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let (socket, _) = self.listener.accept().await?;

            let mut handler = Handler {
                db: self.db_holder.db(),
                connection: Connection::new(socket),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    println!("connection error: {}", err);
                }
                drop(permit);
            });
        }
    }
}

impl Handler {
    pub async fn run(&mut self) -> std::io::Result<()> {
        loop {
            let frame = match self.connection.read_frame().await? {
                Some(f) => f,
                None => {
                    println!("connection closed by peer");
                    return Ok(());
                }
            };

            match Command::from_frame(frame) {
                Ok(Command::Get(cmd)) => {
                    let resp = cmd.apply(&self.db);
                    self.connection.write_frame(&resp).await?;
                }
                Ok(Command::Set(cmd)) => {
                    let resp = cmd.apply(&self.db);
                    self.connection.write_frame(&resp).await?;
                }
                Ok(Command::Publish(cmd)) => {
                    let resp = cmd.apply(&self.db);
                    self.connection.write_frame(&resp).await?;
                }
                Ok(Command::Subscribe(cmd)) => {
                    self.handle_subscribe(cmd).await?;
                    return Ok(());
                }
                Err(e) => {
                    let resp = Frame::Error(format!("ERR {:?}", e));
                    self.connection.write_frame(&resp).await?;
                }
            }
        }
    }

    async fn handle_subscribe(&mut self, subscribe: Subscribe) -> std::io::Result<()> {
        // One mpsc channel collects messages from all broadcast receivers.
        let (tx, mut rx) = mpsc::unbounded_channel::<(String, Bytes)>();

        for (idx, channel) in subscribe.channels().iter().enumerate() {
            let mut bcast_rx = self.db.subscribe(channel.clone());
            let tx = tx.clone();
            let channel_name = channel.clone();

            // Confirm subscription to the client.
            let confirm = Frame::Array(vec![
                Frame::Bulk(Bytes::from("subscribe")),
                Frame::Bulk(Bytes::from(channel.clone())),
                Frame::Integer((idx + 1) as u64),
            ]);
            self.connection.write_frame(&confirm).await?;

            tokio::spawn(async move {
                loop {
                    match bcast_rx.recv().await {
                        Ok(msg) => {
                            if tx.send((channel_name.clone(), msg)).is_err() {
                                break; // receiver (handler) dropped
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            });
        }

        // Drop our sender copy so the channel closes when all spawned tasks end.
        drop(tx);

        while let Some((channel, message)) = rx.recv().await {
            let msg = Frame::Array(vec![
                Frame::Bulk(Bytes::from("message")),
                Frame::Bulk(Bytes::from(channel)),
                Frame::Bulk(message),
            ]);
            self.connection.write_frame(&msg).await?;
        }

        Ok(())
    }
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
