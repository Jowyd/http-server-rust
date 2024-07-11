use itertools::Itertools;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpListener;

#[allow(dead_code)]
fn define_method(method: MethodType) {
    match method {
        MethodType::GET => println!("GET"),
        MethodType::POST => println!("POST"),
        MethodType::PUT => println!("PUT"),
        MethodType::DELETE => println!("DELETE"),
        _ => panic!("Method not supported"),
    }
}

enum ContentType {
    Text,
    Html,
    Json,
}

enum MethodType {
    GET,
    POST,
    PUT,
    DELETE,
}

impl From<&str> for MethodType {
    fn from(method: &str) -> Self {
        match method {
            "GET" => MethodType::GET,
            "POST" => MethodType::POST,
            "PUT" => MethodType::PUT,
            "DELETE" => MethodType::DELETE,
            _ => panic!("Method not supported"),
        }
    }
}

fn format_header_response(response: &Response) -> String {
    let content_type = match response.content_type {
        ContentType::Text => "text/plain",
        ContentType::Html => "text/html",
        ContentType::Json => "application/json",
    };
    let content_length = response.body.len();

    return format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-length: {}\r\n\r\n",
        response.status_code, response.status_message, content_type, content_length
    );
}

struct Response {
    status_code: u8,
    status_message: String,
    content_type: ContentType,
    body: String,
}

impl Response {
    fn to_bytes(&self) -> Vec<u8> {
        let header = format_header_response(&self);
        let body = &self.body;
        return format!("{}{}", header, body).as_bytes().to_vec();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 1024];
                let _ = _stream.read(&mut buffer[..])?;
                println!("received data:\n{}", String::from_utf8_lossy(&buffer[..]));

                let request = String::from_utf8(buffer.into())?;
                let lines = request.lines().collect_vec();
                let start_line = lines.get(0).unwrap();
                let split_start_line = start_line.split(' ').collect_vec();
                let _method = split_start_line.get(0).unwrap().to_string();
                let method_type: MethodType = _method.as_str().into();
                let path = split_start_line.get(1).unwrap();
                let _http_version = split_start_line.get(2).unwrap();

                let host = lines
                    .iter()
                    .find(|line| line.starts_with("Host: "))
                    .unwrap()
                    .split(' ')
                    .collect_vec()
                    .get(1)
                    .unwrap();

                let user_agent_line = lines
                    .iter()
                    .find(|line| line.starts_with("User-Agent: "))
                    .unwrap();

                if path == &"/" || path == &"/index.html" {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                } else if path.starts_with("/echo/") {
                    let (_, data) = path.split_at(6);
                    let response = Response {
                        status_code: 200,
                        status_message: "OK".to_string(),
                        content_type: ContentType::Text,
                        body: data.to_string(),
                    };
                    _stream.write(&response.to_bytes())?;
                } else if path == &"/user-agent" {
                    let (_, user_agent) = user_agent_line.split_at(12);
                    let response = Response {
                        status_code: 200,
                        status_message: "OK".to_string(),
                        content_type: ContentType::Text,
                        body: user_agent.to_string(),
                    };
                    _stream.write(&response.to_bytes())?;
                } else {
                    _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
