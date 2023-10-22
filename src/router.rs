use crate::request::{Method, Request};
use std::{collections::HashMap, io::Write, net::TcpStream};

pub struct Router {
    pub routes: Vec<Route>,
    middleware: Vec<Middleware>,
    not_found: Route,
}

type Params = HashMap<String, String>;

type RouteHandler = fn(&mut TcpStream, Request, RouterState, Params);
type Middleware = fn(&mut TcpStream, &mut Request, RouterState, Params);

type RouterState = Option<HashMap<String, String>>;

#[derive(Clone, PartialEq, Debug)]
pub struct Route {
    pub path: String,
    pub method: Method,
    pub handler: RouteHandler,
}

impl Route {
    pub fn new(path: &str, method: Method, handler: RouteHandler) -> Route {
        Route {
            path: path.to_string(),
            method,
            handler,
        }
    }
}

fn routes_match(route: &Route, request: &Request) -> bool {
    let route_parts: Vec<&str> = route.path.split("/").collect();
    let request_parts: Vec<&str> = request.path.split("/").collect();

    for (route_part, path_part) in route_parts.iter().zip(request_parts.iter()) {
        if !route_part.starts_with(":") && path_part != route_part {
            return false;
        }
    }

    if route.method != request.method {
        return false;
    }

    true
}

fn file_route_handler(stream: &mut TcpStream, request: Request, _state: RouterState, _params: Params) {  
    let file_name = request.path.trim_start_matches("/");
    let path = format!("./src/pages/{}", file_name);
    let contents = std::fs::read_to_string(path).unwrap();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

impl Router {
    pub fn new() -> Router {
        Router {
            routes: Vec::new(),
            not_found: Route {
                path: "".to_string(),
                method: Method::GET,
                handler: not_found,
            },
            middleware: Vec::new(),
        }
    }

    fn sort_routes(&mut self) {
        self.routes.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
    }

    pub fn add_middleware(&mut self, middleware: Middleware) {
        self.middleware.push(middleware);
    }

    pub fn add_route(&mut self, path: &str, method: Method, handler: RouteHandler) {
        self.routes.push(Route {
            path: path.to_string(),
            method,
            handler,
        });

        self.sort_routes();
    }

    fn send_html(&self, stream: &mut TcpStream, html: &str) {
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn register_file_route(&mut self, path: &str) -> anyhow::Result<()> {
        let contents = std::fs::read_to_string(path)?;

        let route_handler = file_route_handler;

        let path = path.trim_start_matches("./src/pages");

        self.add_route(path, Method::GET, route_handler);

        Ok(())
    }

    pub fn load_file_routes(&mut self, path: &str) -> anyhow::Result<()> {
        let entries = std::fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                continue;
            }


            self.register_file_route(entry.path().to_str().unwrap())?;
        }

        Ok(())
    }

    fn get_route(&self, request: &Request) -> anyhow::Result<Route> {
        for route in &self.routes {
            if routes_match(route, request) {
                return Ok(route.clone());
            }
        }

        return Err(anyhow::anyhow!("No route found"));
    }

    fn get_handler(&self, request: &Request) -> anyhow::Result<RouteHandler> {
        for route in &self.routes {
            if routes_match(route, request) {
                return Ok(route.handler);
            }
        }

        return Err(anyhow::anyhow!("No route found"));
    }

    pub fn listen(&self, port: Option<String>) -> anyhow::Result<()> {
        let url = format!("0.0.0.0:{}", port.unwrap_or("3000".to_string()));
        let listener = std::net::TcpListener::bind(url)?;

        loop {
            // The second item contains the IP and port of the new connection.
            let (mut socket, _) = listener.accept()?;

            // Get the request from the socket.
            let mut request = Request::from_stream(&mut socket)?;

            for middleware in &self.middleware {
                middleware(&mut socket, &mut request, None, HashMap::new());
            }

            // Get the Route and the RouteHandler for the request.
            let route = match self.get_route(&request) {
                Ok(route) => route,
                Err(_) => {
                    (self.not_found.handler)(&mut socket, request, None, HashMap::new());
                    continue;
                }
            };

            let handler = self.get_handler(&request)?;

            // Get the params for the request.
            let params = request.get_params(&route);

            // Spawn a new thread to deal with the request.
            std::thread::spawn(move || {
                handler(&mut socket, request, None, params);
            });
        }
    }
}

fn not_found(
    stream: &mut TcpStream,
    _request: Request,
    _state: RouterState,
    _params: Params,
) {
    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
