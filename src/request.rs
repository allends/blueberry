use std::io::Write;
use std::{collections::HashMap, io::Read};
use std::net::TcpStream;

use crate::{router::Route};

#[derive(Clone, PartialEq, Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Clone, PartialEq, Debug)]
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

    pub fn send_message(&self, stream: &mut TcpStream, message: &str) {
        stream.write(message.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    pub fn from_stream(stream: &mut TcpStream) -> anyhow::Result<Request> {
        let mut buffer: [u8; 512] = [0u8; 512];
        let bytes_read = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        Ok(Request::from(&request))
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

    pub fn get_params(&self, route: &Route) -> HashMap<String, String> {
        let mut result = HashMap::new();

        let route_parts: Vec<&str> = route.path.split("/").collect();
        let request_parts: Vec<&str> = self.path.split("/").collect();

        let mut current = 0;
        let mut trailing: Option<String> = None;
        let length = route_parts.len();

        while current < length {
            let route_part = route_parts.get(current).unwrap();
            let path_part = request_parts.get(current).unwrap();
            if route_part.starts_with(":") {
                let var_name = &route_part[1..];
                result.insert(var_name.to_string(), path_part.to_string());
                trailing = Some(var_name.to_string());
            } else {
                trailing = None;
            }
            current += 1;
        }

        if let Some(key) = trailing {
            if current >= request_parts.len() {
                return result;
            }
            let mut modifying = result.remove(&key).unwrap();
            while current < request_parts.len() {
                let adding = request_parts.get(current).unwrap();
                modifying.push('/');
                modifying.push_str(adding);
                current += 1;
            }
            result.insert(key, modifying.clone());
        }

        return result;
    }

    fn get_body(request: &str) -> &str {
        request.split("\r\n").last().unwrap()
    }
}
