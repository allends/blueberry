// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path;

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

struct Router {
    routes: Vec<Route>,
}

struct Route {
    path: String,
    handler: fn(&mut TcpStream, String),
}

impl Router {
    fn new() -> Router {
        Router { routes: Vec::new() }
    }

    fn sort_routes(&mut self) {
        self.routes.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    }

    fn add_route(&mut self, path: &str, handler: fn(&mut TcpStream, String)) {
        self.routes.push(Route {
            path: path.to_string(),
            handler,
        });

        self.sort_routes();
    }

    fn route_request(&self, stream: &mut TcpStream) {
        let request = parse_request(stream);
        let path = get_path(&request);
        for route in &self.routes {
            let index = path.find(route.path.as_str()).unwrap_or(1);
            println!("{} {} {}", path, route.path, index);
            if index == 0 {
                (route.handler)(stream, request);
                return;
            }
        }
        send_message(stream, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let mut router = Router::new();

    router.add_route("/", |stream, _| {
        send_message(stream, "HTTP/1.1 200 OK\r\n\r\n");
    });

    router.add_route("/echo", |stream, request| {
        let path = get_path(&request);
        let payload = path.split("/echo").nth(1).unwrap_or("");
        let message = format!("HTTP/1.1 200 OK\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}", payload.len(), payload);
        println!("{}", message);
        send_message(stream, &message);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                router.route_request(&mut _stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
