use std::collections::HashMap;
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

fn get_headers(request: String) -> HashMap<String, String> {
    let mut headers: HashMap<String, String> = HashMap::new();
    let lines = request.split("\r\n");

    for line in lines {
        if line.contains(":") {
            let parts: Vec<&str> = line.split(":").collect();
            let key = parts[0].to_string();
            let value = parts[1].to_string().trim().to_owned();
            headers.insert(key, value);
        }
    }

    return headers;
}

fn not_found(stream: &mut TcpStream) {
    send_message(stream, "HTTP/1.1 404 Not Found\r\n\r\n");
}

type RouteState = Option<HashMap<String, String>>;
type Params = HashMap<String, String>;
type RouteHandler = fn(&mut TcpStream, String, RouteState, Params);

#[derive(Clone)]
struct Router {
    routes: Vec<Route>,
}

#[derive(Clone)]
struct Route {
    path: String,
    handler: RouteHandler,
    state: Option<HashMap<String, String>>,
}

impl Router {
    fn new() -> Router {
        Router { routes: Vec::new() }
    }

    fn sort_routes(&mut self) {
        self.routes.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    }

    fn add_route(&mut self, path: &str, handler: RouteHandler, state: Option<HashMap<String, String>>) {
        self.routes.push(Route {
            path: path.to_string(),
            handler,
            state,
        });

        self.sort_routes();
    }

    fn route_request(&self, stream: &mut TcpStream) {
        let request = parse_request(stream);
        let path = get_path(&request);

        for route in &self.routes {
            if routes_match(path, route.path.as_str()) {
                let params = get_params(path, &route.path);
                (route.handler)(stream, request, route.state.clone(), params);
                return;
            }

        }
        send_message(stream, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}

fn routes_match(path: &str, route: &str) -> bool {
    let path_parts: Vec<&str> = path.split("/").collect();
    let route_parts: Vec<&str> = route.split("/").collect();

    for (route_part, path_part) in route_parts.iter().zip(path_parts.iter()) {
        if !route_part.starts_with(":") && path_part != route_part {
            return false;
        }
    }

    true
}

fn get_params(path: &str, route: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let path_parts: Vec<&str> = path.split("/").collect();
    let route_parts: Vec<&str> = route.split("/").collect();

    let mut current = 0;
    let mut trailing: Option<String> = None;
    let length = route_parts.len();

    while current < length {
        let path_part = path_parts.get(current).unwrap();
        let route_part = route_parts.get(current).unwrap();
        if route_part.starts_with(":") {
            let var_name = &route_part[1..];
            result.insert(var_name.to_string(), path_part.to_string());
            trailing = Some(var_name.to_string());
        } else{
            trailing = None;
        }
        current += 1;
    }

    if let Some(key) = trailing {
        if current >= path_parts.len() {
            return result;
        }
        let mut modifying = result.remove(&key).unwrap();
        while current < path_parts.len() {
            let adding = path_parts.get(current).unwrap();
            modifying.push('/');
            modifying.push_str(adding);
            current += 1;
        }
        result.insert(key, modifying.clone());
    }

    return result;
}

#[tokio::main]
async fn main()  -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let mut router = Router::new();

    router.add_route("/", |stream, _, _, _| {
        send_message(stream, "HTTP/1.1 200 OK\r\n\r\n");
    }, None);

    router.add_route("/echo/:input", |stream, request, _, _| {
        let path = get_path(&request);
        let payload = path.split("/echo").nth(1).unwrap_or("").to_owned();
        let trimmed = &payload[1..];
        let message = format!("HTTP/1.1 200 OK\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}", trimmed.len(), trimmed);
        send_message(stream, &message);
    }, None); 

    router.add_route("/user-agent", |stream, request, _, _| {
        let headers = get_headers(request);
        let user_agent = headers.get("User-Agent").unwrap();
        let message = format!("HTTP/1.1 200 OK\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}", user_agent.len(), user_agent);
        println!("{}", message);
        send_message(stream, &message);
    }, None);

    
    let dir = std::env::args().nth(3);

    if let Some(target_dir) = dir {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("dir".to_string(), target_dir);
        router.add_route("/files/:path", |stream, request, state, params| {
            let state_dict = state.unwrap();
            let dir_string = state_dict.get("dir").unwrap();
            let path = params.get("path").unwrap();
            let file = std::fs::File::open(dir_string.to_owned() + path);
            match file {
                Ok(suc) => {
                    println!("{:?}", suc);
                },
                Err(_) => not_found(stream),
            }
        }, Some(map));
    }

    for stream in listener.incoming() {
        let other_router = router.clone();
        tokio::task::spawn(async move {
            match stream {
                Ok(mut _stream) => {
                    println!("accepted new connection");
                    other_router.route_request(&mut _stream);
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        });
        
    }

    return Ok(());
}
