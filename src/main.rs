// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn send_message(stream: &mut TcpStream, message: &str) {
    stream.write(message.as_bytes()).unwrap();
}

fn get_path(request: &str) -> &str {
    let lines = request.split("\r\n");
    let mut path = "";
    for line in lines {
        if line.starts_with("GET") {
            path = line.split(" ").nth(1).unwrap();
        }
    }
    path
}

fn parse_request(stream: &mut TcpStream) -> String {
    let mut buffer: [u8; 512] = [0u8; 512];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    return request.to_string();
}

fn echo_request(stream: &mut TcpStream, request: String) {
    let path = get_path(&request);
    let payload = path.split("/").nth(3).unwrap_or("");
    let message = format!("
    HTTP/1.1 200 OK\r\n\
    Content-Type: text/plain\r\n\
    Content-Length: {}\r\n\
    \r\n\
    {}
    /r/n/r/n", payload.len(), payload);
    send_message(stream, &message);
}

fn route_request(stream: &mut TcpStream) {
    let request = parse_request(stream);
    let path = get_path(&request).split("/").take(2).collect::<Vec<&str>>().join("");
    let path = path.as_str();
    println!("path: {}", path);
    match path {
        "/" => send_message(stream, "HTTP/1.1 200 OK\r\n\r\n"),
        "/echo" => echo_request(stream, request),
        _ => send_message(stream, "HTTP/1.1 404 Not Found\r\n\r\n"),
    }
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
                route_request(&mut _stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
