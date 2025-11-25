use clap::Parser;
use std::net::SocketAddr;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    sync::broadcast,
};
use tracing::{error, info};

/// Defines the command-line arguments for the P2P chat application.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The address and port to listen for incoming connections on.
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    pub listen_addr: SocketAddr,

    /// A list of bootstrap peer addresses to connect to on startup.
    /// Can be specified multiple times.
    #[arg(short, long = "connect")]
    pub connect_addrs: Vec<SocketAddr>,
}

/// Handles reading user input from STDIN.
///
/// Reads lines from stdin and broadcasts them on the `chat_tx` channel.
/// Shuts down when a message is received on `shutdown_rx`.
pub async fn handle_stdin(
    chat_tx: broadcast::Sender<String>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut stdin = BufReader::new(io::stdin()).lines();
    info!("Enter messages to chat. Type 'exit' or press Ctrl+D to quit.");

    loop {
        tokio::select! {
            // Listen for the shutdown signal
            _ = shutdown_rx.recv() => {
                info!("STDIN task shutting down.");
                break;
            }
            // Read a line from stdin
            result = stdin.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        if line == "exit" {
                            break;
                        }
                        // Broadcast the message to all peers
                        if let Err(e) = chat_tx.send(line.to_string()) {
                            error!("Failed to broadcast message: {}", e);
                        }
                    }
                    Ok(None) => {
                        // STDIN closed (Ctrl+D)
                        info!("STDIN closed.");
                        break;
                    }
                    Err(e) => {
                        error!("Error reading from STDIN: {}", e);
                        break;
                    }
                }
            }
        }
    }
}