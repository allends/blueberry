# Blueberry

A minimal framework to write HTTP servers.

Example app
```rust
use request::Method;
use router::Router;

mod request;
mod router;

// This is the function signature of an endpoint
fn test_endpoint(
    stream: &mut std::net::TcpStream,
    request: request::Request,
    _state: Option<std::collections::HashMap<String, String>>,
    params: std::collections::HashMap<String, String>,
) {
    let id = params.get("id").unwrap();
    let message = format!(
        "HTTP/1.1 200 OK\r\n\
    Content-Type: text/plain\r\n\
    Content-Length: {}\r\n\
    \r\n\
    {}",
        id.len(),
        id
    );
    request.send_message(stream, &message);
}

fn main() -> anyhow::Result<()> {
   let mut router = Router::new();

   // add routes via closure
   router.add_route("/", Method::GET, |stream, request, _, _| {
      request.send_message(stream, "HTTP/1.1 200 OK\r\n\r\n");
   });

   // add routes via handler
   router.add_route("/allen/:id", Method::GET, test_endpoint);

   // middleware runs before each request
   router.add_middleware(|_, request, _, _| {
      println!("got request: {:#?}", request);
   });

   // load static html or other files you want to serve
   router.load_file_routes(path.unwrap_or(&"./src/pages".to_string()))?;

   // listen for clients to serve
   router.listen(Some("3000".to_string()))
}
```