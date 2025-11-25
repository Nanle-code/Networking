Rust P2P Chat

This is a mini P2P (peer-to-peer) chat application built in Rust, as part of a learning project. It allows multiple users to connect to each other and chat over a TCP network without a central server.

Features

Decentralized: No central server required. Peers connect directly to each other.

CLI Interface: Simple command-line interface for sending and receiving messages.

Peer Discovery: Basic peer discovery via a hardcoded bootstrap list on startup.

Async Networking: Uses tokio for high-performance, asynchronous network I/O.

Graceful Shutdown: Shuts down all connections and tasks cleanly on Ctrl+C.

Project Structure

src/main.rs: The main entry point. Sets up tokio, logging, CLI parsing, and all the main tasks (stdin, stdout, listener, connector). Manages the graceful shutdown process.

src/cli.rs: Handles parsing clap command-line arguments and the async task for reading lines from stdin.

src/peer.rs: Contains the handle_connection function, which is the core logic for managing a single peer connection. It's spawned as a new task for every peer.

Cargo.toml: The project manifest, defining dependencies like tokio, clap, tracing, and anyhow.

How It Works

The application uses a "gossip" or "broadcast" model.

Channels:

A tokio::sync::broadcast channel (chat_tx) is used to send messages from your STDIN to all connected peers.

A tokio::sync::mpsc channel (msg_tx) is used to send messages from all peers to your STDOUT for printing.

Tasks: The main function spawns several long-running async tasks:

STDIN Task (cli::handle_stdin): Reads lines from your terminal. When you type a message and press Enter, it sends this message into the chat_tx (broadcast) channel.

STDOUT Task: A simple loop in main that listens on the msg_rx (mpsc) channel and prints any received messages to your terminal.

Listener Task: Binds to the --listen-addr and accepts new incoming TcpStream connections. For each new peer, it spawns a peer::handle_connection task.

Connector Task: Reads the list of --connect addresses and attempts to connect to each one. For each successful connection, it also spawns a peer::handle_connection task.

Peer Connection (peer::handle_connection): This is the heart of the app. Each peer connection runs in its own task.

It subscribes to the chat_tx (broadcast) channel.

It gets a clone of the msg_tx (mpsc) sender.

It uses tokio::select! to concurrently:

Wait for a message from chat_rx (from your STDIN) and write it to the peer's TcpStream.

Wait for a message from the peer's TcpStream, prepend the peer's address, and send it to msg_tx (to be printed on your STDOUT).

Wait for a global shutdown signal to terminate the connection.

How to Run

Build the project:

cargo build --release


Run the first peer (the "bootstrap" node):
Open a terminal and run:

./target/release/rust-p2p-chat --listen 127.0.0.1:8080


This node will just listen.

Run a second peer:
Open a second terminal and run:

./target/release/rust-p2p-chat --listen 127.0.0.1:8081 --connect 127.0.0.1:8080


This node will listen on port 8081 and also connect to the first node on port 8080.

Chat!
Type messages in either terminal and press Enter. You should see the messages appear in the other terminal, prefixed by the sender's address.

Run a third peer (optional):
Open a third terminal:

./target/release/rust-p2p-chat --listen 127.0.0.1:8082 --connect 127.0.0.1:8081


This node connects to the second node. Now, try typing in any terminal. Because of the broadcast model, the message will be sent to all connected peers. (Note: This simple model isn't a true "mesh"; this third peer won't be connected to the first peer unless you add it manually. A "true" P2P app would exchange peer lists).