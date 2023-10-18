use std::{collections::HashMap, net::{TcpStream, TcpListener}};
use crate::request::{Request, Method};

struct Router {
    routes: Vec<Route>,
}

type Params = HashMap<String, String>;
type RouteState = Option<HashMap<String, String>>;

type RouteHandler = fn(&mut TcpStream, Request, RouteState, Params);

struct Route {
    path: String,
    method: Method,
    handler: RouteHandler,
}

impl Router {
  fn new() -> Router {
    Router { routes: Vec::new() }
  }

  fn sort_routes(&mut self) {
    self.routes.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
  }

  fn add_route(&mut self, path: &str, method: Method, handler: RouteHandler) {
    self.routes.push(Route {
      path: path.to_string(),
      method,
      handler,
    });

    self.sort_routes();
  }

  fn listen(&self, port: &str) {
    let listener = TcpListener::bind("0.0.0.0:0000").unwrap();
    for stream in listener.incoming() {
      let cloned_router = self.clone();
      std::thread::spawn(move || {
        match stream {
          Ok(mut stream) => {
            // cloned_router.route_request(&mut stream);
          }
          Err(e) => {
            println!("Unable to connect: {}", e);
          }
        }
      });
    }
  }
}