// Uncomment this block to pass the first stage
use std::net::{TcpListener, TcpStream};
use std::io::{ Read, Write };

fn send_message(stream: &mut TcpStream, message: &str) {
    stream.write(message.as_bytes()).unwrap();
}



fn parse_request(stream: &mut TcpStream) {
    let mut buffer: [u8; 512] = [0u8; 512];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("request: {}", request);
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer: [u8; 512] = [0u8; 512];
                let bytes_read = _stream.read(&mut buffer).unwrap();
                send_message(&mut _stream, "HTTP/1.1 200 OK\r\n\r\n");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
