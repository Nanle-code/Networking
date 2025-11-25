use futures::StreamExt;
use std::net::SocketAddr;
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info, warn};

/// Handles a single, established peer connection.
///
/// This function is spawned in a new task for each connection.
/// It uses a `Framed` stream with `LinesCodec` to send/receive newline-delimited
/// strings (i.e., chat messages).
///
/// It concurrently:
/// 1. Reads messages from its `chat_rx` (messages from our STDIN) and writes them to the socket.
/// 2. Reads messages from the socket and sends them to `msg_tx` (to be printed to our STDOUT).
/// 3. Listens for a shutdown signal on `shutdown_rx`.
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    mut chat_rx: broadcast::Receiver<String>,
    msg_tx: mpsc::Sender<String>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    info!("Handling connection for peer: {}", addr);

    // Use a `LinesCodec` to frame the TCP stream into newline-delimited lines.
    // `Framed` gives us a `Stream` (for reading) and a `Sink` (for writing).
    let mut framed = Framed::new(stream, LinesCodec::new());

    loop {
        tokio::select! {
            // === 1. Listen for shutdown signal ===
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received, closing connection with {}", addr);
                break;
            }

            // === 2. Receive messages from our STDIN (via broadcast) and send to peer ===
            result = chat_rx.recv() => {
                match result {
                    Ok(msg) => {
                        // Prepend our address (or a username) to the message
                        let msg_to_send = format!("[You]: {}", msg);
                        // Send the message to the peer
                        if let Err(e) = framed.send(msg_to_send).await {
                            error!("Failed to send message to peer {}: {}", addr, e);
                            break; // Stop on write error
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Message broadcast lagged for peer {}, missed {} messages", addr, n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // This should technically not happen if main task is alive
                        error!("Chat broadcast channel closed.");
                        break;
                    }
                }
            }

            // === 3. Receive messages from peer (from socket) and send to STDOUT (via mpsc) ===
            result = framed.next() => {
                match result {
                    Some(Ok(line)) => {
                        // We received a line from the peer.
                        // Prepend their address and send it to the stdout task.
                        let msg_to_print = format!("[{addr}]: {line}");
                        if msg_tx.send(msg_to_print).await.is_err() {
                            // This means the stdout task has died, which is a fatal error.
                            error!("Failed to send received message to STDOUT task. Shutting down connection.");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("Error reading from peer {}: {}", addr, e);
                        break; // Stop on read error
                    }
                    None => {
                        // Stream closed by the peer
                        info!("Peer {} disconnected.", addr);
                        break;
                    }
                }
            }
        }
    }
    info!("Connection handler for {} finished.", addr);
}