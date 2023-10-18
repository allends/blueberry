use std::{collections::HashMap, io::Read};

pub struct Request {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

pub enum Method {
    GET,
    POST,
}

impl Request {
    pub fn from(request: &str) -> Request {
        let method = Request::get_method(request);
        let path = Request::get_path(request).to_string();
        let headers = Request::get_headers(request);
        let body = Request::get_body(request).to_string();

        Request {
            method,
            path,
            headers,
            body,
        }
    }

    pub fn from_stream(stream: &mut std::net::TcpStream) -> Request {
        let mut buffer: [u8; 512] = [0u8; 512];
        let bytes_read = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        Request::from(&request)
    }

    fn get_method(request: &str) -> Method {
        let main_line = request.split("\r\n").into_iter().next().unwrap();
        let method = main_line.split(" ").into_iter().next().unwrap();

        match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            _ => Method::GET,
        }
    }

    fn get_path(request: &str) -> &str {
        let lines = request.split("\r\n");
        let main_line = lines.into_iter().next().unwrap();
        main_line.split(" ").nth(1).unwrap()
    }

    fn get_headers(request: &str) -> HashMap<String, String> {
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

    fn get_body(request: &str) -> &str {
        request.split("\r\n").last().unwrap()
    }
}
