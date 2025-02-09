use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

// const ADDRESS: &str = "127.0.0.1:6974";
// const RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
