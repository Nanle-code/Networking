use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    signal,
    sync::{broadcast, mpsc},
};
use tracing::{error, info};

mod cli;
mod peer;

/// The main entry point for the P2P chat application.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (logging)
    tracing_subscriber::fmt::init();

    // Parse command-line arguments
    let args = cli::Args::parse();
    info!("Starting P2P chat node with args: {:?}", args);

    // --- Channels ---
    // 1. For broadcasting chat messages to all connected peers
    let (chat_tx, _) = broadcast::channel(100);
    // 2. For sending messages received from peers to the main stdout printer
    let (msg_tx, mut msg_rx) = mpsc::channel(100);

    // --- Graceful Shutdown ---
    // A channel to signal all tasks to shut down.
    let (shutdown_tx, _) = broadcast::channel(1);

    // --- Task: Handle STDIN ---
    // Reads lines from stdin and broadcasts them on `chat_tx`.
    let stdin_task = tokio::spawn(cli::handle_stdin(
        chat_tx.clone(),
        shutdown_tx.subscribe(),
    ));

    // --- Task: Print Received Messages ---
    // Receives messages on `msg_rx` and prints them to stdout.
    let stdout_task = tokio::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            println!("{}", msg);
        }
    });

    // --- Task: Listen for Incoming Connections ---
    let listener = TcpListener::bind(&args.listen_addr).await?;
    info!("Listening for connections on {}", args.listen_addr);
    let listen_task = tokio::spawn(async move {
        let chat_tx = chat_tx.clone();
        let msg_tx = msg_tx.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();

        loop {
            tokio::select! {
                // Wait for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Listener task shutting down.");
                    break;
                }
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("New connection from: {}", addr);
                            // Spawn a new task for each connection
                            tokio::spawn(peer::handle_connection(
                                stream,
                                addr,
                                chat_tx.subscribe(),
                                msg_tx.clone(),
                                shutdown_tx.subscribe(),
                            ));
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }
    });

    // --- Task: Connect to Bootstrap Peers ---
    // We use Arc to share the list without cloning the vec itself
    let bootstrap_peers = Arc::new(args.connect_addrs);
    let connect_task = tokio::spawn(async move {
        let chat_tx = chat_tx.clone();
        let msg_tx = msg_tx.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();

        // Try to connect to each bootstrap peer
        for addr in bootstrap_peers.iter() {
            info!("Attempting to connect to bootstrap peer: {}", addr);
            match tokio::net::TcpStream::connect(addr).await {
                Ok(stream) => {
                    info!("Successfully connected to peer: {}", addr);
                    tokio::spawn(peer::handle_connection(
                        stream,
                        *addr,
                        chat_tx.subscribe(),
                        msg_tx.clone(),
                        shutdown_tx.subscribe(),
                    ));
                }
                Err(e) => {
                    error!("Failed to connect to peer {}: {}", addr, e);
                }
            }
        }
        
        // Keep this task alive to listen for shutdown
        // This is a simple way; a more robust app might have peer retry logic here
        let _ = shutdown_rx.recv().await;
        info!("Connect task shutting down.");
    });

    // --- Wait for Shutdown Signal ---
    // Wait for Ctrl+C
    info!("Press Ctrl+C to shut down.");
    signal::ctrl_c().await?;

    info!("Ctrl+C received, sending shutdown signal...");

    // Send the shutdown signal to all tasks.
    // The `_` ignores the error in case there are no subscribers.
    let _ = shutdown_tx.send(());

    // --- Await Task Completion ---
    // Wait for all main tasks to complete.
    // We can ignore results here as errors should be handled internally.
    let _ = tokio::join!(
        stdin_task,
        stdout_task,
        listen_task,
        connect_task
    );

    info!("Shutdown complete.");
    Ok(())
}