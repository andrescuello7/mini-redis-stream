mod server;
mod connection;
mod db;
mod parse;
mod cmd;
use server::run;

// This is Redis STM, a simple implementation of a Redis server using Rust and Tokio.
// The main function initializes the server and starts listening for incoming connections.
// The `#[tokio::main]` attribute is a macro that sets up the Tokio runtime and allows us to write asynchronous code in the main function.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[*] Starting Redis STM...");
    run().await?;
    Ok(())
}
