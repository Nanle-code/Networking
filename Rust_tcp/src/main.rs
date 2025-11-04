use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    // A buffer to read data from the clients
    let mut buffer: [u8; 1024] = [0; 1024];
    // This line reads data from the client into the buffer
    stream.read(&mut buffer).expect("Failed to read from client");

    // this line converts the data in the buffer to a string
    let request = String::from_utf8_lossy(&buffer);
    println!("Received request: {}", request);

    let response = "Hello, Client!".as_bytes();
    stream.write(response).expect("Failed to write to client");


    // Echo the received data back to the client
}

fn main(){
    // Bind the TCP listener to a specific address and port
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:8080");
}