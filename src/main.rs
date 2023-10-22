use request::Method;
use router::Router;

mod request;
mod router;

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
    let argvec = std::env::args().collect::<Vec<String>>();
    let mut argiter = argvec.iter();
    argiter.find(|arg| arg.as_str().eq("--directory"));
    let path = argiter.next();

    let mut router = Router::new();

    router.add_route("/", Method::GET, |stream, request, _, _| {
        request.send_message(stream, "HTTP/1.1 200 OK\r\n\r\n");
    });

    router.add_route("/allen/:id", Method::GET, test_endpoint);

    router.add_middleware(|_, request, _, _| {
        println!("got request: {:#?}", request);
    });

    router.load_file_routes(path.unwrap_or(&"./src/pages".to_string()))?;

    println!("{:?}", router.routes);

    router.listen(Some("3000".to_string()))
}
