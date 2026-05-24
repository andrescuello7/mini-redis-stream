use tokio::net::TcpListener;
use tokio::sync::{Semaphore};
use std::sync::Arc;

use crate::cmd::Command;
use crate::connection::Connection;
use crate::db::{Db, DbDropGuard};

// The Listener struct is responsible for managing the TCP listener and the connection limit.
// It encapsulates the TcpListener and the maximum number of concurrent connections allowed.
pub struct Listener {
    /// Shared database handle.
    ///
    /// Contains the key / value store as well as the broadcast channels for
    /// pub/sub.
    ///
    /// This holds a wrapper around an `Arc`. The internal `Db` can be
    /// retrieved and passed into the per connection state (`Handler`).
    db_holder: DbDropGuard,
    
    // TCP listener that will accept incoming connections.
    listener: TcpListener,

    /// Limit the max number of connections.
    ///
    /// A `Semaphore` is used to limit the max number of connections. Before
    /// attempting to accept a new connection, a permit is acquired from the
    /// semaphore. If none are available, the listener waits for one.
    ///
    /// When handlers complete processing a connection, the permit is returned
    /// to the semaphore.
    limit_connections: Arc<Semaphore>,
}

// The address and port the server will listen on. 
// In this case, it's set to localhost on port 6379, which is the default port for Redis.
const STRING_CONNECTION: &str = "127.0.0.1:6379";

// The maximum number of concurrent connections the server will accept.
// This is a simple way to prevent the server from being overwhelmed by too many clients.
// In a real-world application, you might want to implement a more sophisticated connection management strategy,
const MAX_CONNECTIONS: usize = 250;

struct Handler {
    /// Shared database handle.
    ///
    /// When a command is received from `connection`, it is applied with `db`.
    /// The implementation of the command is in the `cmd` module. Each command
    /// will need to interact with `db` in order to complete the work.
    db: Db,
    
    /// The TCP connection decorated with the redis protocol encoder / decoder
    /// implemented using a buffered `TcpStream`.
    ///
    /// When `Listener` receives an inbound connection, the `TcpStream` is
    /// passed to `Connection::new`, which initializes the associated buffers.
    /// `Connection` allows the handler to operate at the "frame" level and keep
    /// the byte level protocol parsing details encapsulated in `Connection`.
    connection: Connection,
}

// The `server` function takes an address and a connection limit as parameters, binds a TcpListener to the specified address, and returns a new Listener instance.
pub async fn run() -> std::io::Result<Listener> {
    // When the provided `shutdown` future completes, we must send a shutdown
    // message to all active connections. We use a broadcast channel for this
    // purpose. The call below ignores the receiver of the broadcast pair, and when
    // a receiver is needed, the subscribe() method on the sender is used to create
    // one.
    let listener: Listener = Listener {
        db_holder: DbDropGuard::new(),
        listener: TcpListener::bind(STRING_CONNECTION).await?,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    // Initialize the listener state
    let mut server: Listener = listener;

    tokio::select! {
        // Wait for the server to start listening for incoming connections.
        // This is done by awaiting the `run` method on the Listener struct, which will block until the server is ready to accept connections.
        res = server.run() => {
            // If an error is received here, accepting connections from the TCP
            // listener failed multiple times and the server is giving up and
            // shutting down.
            //
            // Errors encountered when handling individual connections do not
            // bubble up to this point.
            if let Err(err) = res {
                println!("failed to accept connections: {}", err);
            }
        },
    }
    println!("Listening on {}", server.listener.local_addr()?);
    Ok(Listener { listener: server.listener, limit_connections: server.limit_connections, db_holder: server.db_holder })
}

// The Listener struct has an associated function `server` that creates a new instance of Listener.
impl Listener {
    /// Run the server
    ///
    /// Listen for inbound connections. For each inbound connection, spawn a
    /// task to process that connection.
    
    // The `server` function takes an address and a connection limit as parameters, binds a TcpListener to the specified address, and returns a new Listener instance.
    pub async fn run(&mut self) -> Result<Vec<u8>> {
        println!("accepting inbound connections");

        loop {
            // Acquire a permit from the semaphore before accepting a new connection.
            // This ensures that we do not exceed the maximum number of concurrent connections.
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let (socket, _) = self.listener.accept().await?;

            let mut handler = Handler {
                // Get a handle to the shared database.
                db: self.db_holder.db(),

                // Initialize the connection state. This allocates read/write
                // buffers to perform redis protocol frame parsing.
                connection: Connection::new(socket),
            };

            // Spawn a new task to handle the connection. The permit is moved into the task, and will be automatically released when the task completes.
            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    println!("connection error : {}", err);
                }
                // Handle the connection here. For example, you could read from the socket, process commands, and write responses back to the client.
                // The permit will be automatically released when this task completes, allowing another connection to be accepted.
                drop(permit); // Explicitly drop the permit to release it back to the semaphore.
            });
        }
    }
}

impl Handler {
    /// Process a single connection.
    ///
    /// Request frames are read from the socket and processed. Responses are
    /// written back to the socket.
    ///
    /// When the shutdown signal is received, the connection is processed until
    /// it reaches a safe state, at which point it is terminated.
    pub async fn run(&mut self) -> std::io::Result<Option<String>> {
        loop {
            if let Some(frame) = self.connection.read_frame().await? {
                let _ = Command::from_frame(frame);

                // TODO i18n: parse the frame into a command and execute it against the database.
                // let response = cmd.unwrap_or_else(|err| format!("Error: {}", err));
                // self.connection.write(response.as_bytes()).await?;
                // return Ok(Some(frame));
            } else {
                println!("Err: read the frame socket closed by peer");
                return Ok(None);
            }
        }
    }
}

/// Error returned by most functions.
///
/// When writing a real application, one might want to consider a specialized
/// error handling crate or defining an error type as an `enum` of causes.
/// However, for our example, using a boxed `std::error::Error` is sufficient.
///
/// For performance reasons, boxing is avoided in any hot path. For example, in
/// `parse`, a custom error `enum` is defined. This is because the error is hit
/// and handled during normal execution when a partial frame is received on a
/// socket. `std::error::Error` is implemented for `parse::Error` which allows
/// it to be converted to `Box<dyn std::error::Error>`.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
