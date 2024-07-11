use itertools::Itertools;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::{fs, thread};

#[allow(dead_code)]
fn define_method(method: MethodType) {
    match method {
        MethodType::GET => println!("GET"),
        MethodType::POST => println!("POST"),
        MethodType::PUT => println!("PUT"),
        MethodType::DELETE => println!("DELETE"),
    }
}

#[allow(dead_code)]
enum ContentType {
    Text,
    Html,
    Json,
    OctetStream,
}

enum MethodType {
    GET,
    POST,
    PUT,
    DELETE,
}

trait MethodHandler {
    fn handle(&self, path: &str) -> Response;
}

impl From<&str> for MethodType {
    fn from(method: &str) -> MethodType {
        match method {
            "GET" => MethodType::GET,
            "POST" => MethodType::POST,
            "PUT" => MethodType::PUT,
            "DELETE" => MethodType::DELETE,
            _ => panic!("Method not supported"),
        }
    }
}

impl FromStr for MethodType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(MethodType::GET),
            "POST" => Ok(MethodType::POST),
            "PUT" => Ok(MethodType::PUT),
            "DELETE" => Ok(MethodType::DELETE),
            _ => Err(format!("Invalid HTTP method: {}", s)),
        }
    }
}

fn format_header_response(response: &Response) -> String {
    let content_type = match response.content_type {
        ContentType::Text => "text/plain",
        ContentType::Html => "text/html",
        ContentType::Json => "application/json",
        ContentType::OctetStream => "application/octet-stream",
    };
    let content_length = response.body.len();

    return format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-length: {}\r\n\r\n",
        response.status_code, response.status_message, content_type, content_length
    );
}

struct Response {
    status_code: u16,
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

    fn not_found() -> Response {
        Response {
            status_code: 404,
            status_message: "Not Found".to_string(),
            content_type: ContentType::Text,
            body: "Not Found".to_string(),
        }
    }
}

struct Request {
    method: MethodType,
    path: String,
    http_version: String,
    host: String,
    user_agent: String,
    accept: String,
    body: String,
}

impl Request {
    fn parse(request: &str) -> Result<Request, String> {
        let mut lines = request.lines();

        // Parse la première ligne
        let first_line = lines.next().ok_or("Request is empty")?;
        let mut parts = first_line.split_whitespace();
        let method = parts.next().ok_or("Missing method")?.parse()?;
        let path = parts.next().ok_or("Missing path")?.to_string();
        let http_version = parts.next().ok_or("Missing HTTP version")?.to_string();

        let mut host = String::new();
        let mut user_agent = String::new();
        let mut accept = String::new();
        let mut body = String::new();
        let mut headers_ended = false;

        // Parse les en-têtes et le corps
        for line in lines {
            if line.is_empty() && !headers_ended {
                headers_ended = true;
                continue;
            }

            if !headers_ended {
                let mut header_parts = line.splitn(2, ": ");
                let header_name = header_parts.next().unwrap_or("");
                let header_value = header_parts.next().unwrap_or("");

                match header_name.to_lowercase().as_str() {
                    "host" => host = header_value.to_string(),
                    "user-agent" => user_agent = header_value.to_string(),
                    "accept" => accept = header_value.to_string(),
                    _ => {} // Ignorer les autres en-têtes
                }
            } else {
                // Ajouter la ligne au corps
                body.push_str(line);
                body.push('\n');
            }
        }

        // Supprimer le dernier caractère newline du corps s'il existe
        if body.ends_with('\n') {
            body.pop();
        }

        Ok(Request {
            method,
            path,
            http_version,
            host,
            user_agent,
            accept,
            body,
        })
    }

    fn handle(&self) -> Response {
        match self.method {
            MethodType::GET => self.handle_get(),
            MethodType::POST => self.handle_post(),
            _ => Response {
                status_code: 405,
                status_message: "Method Not Allowed".to_string(),
                content_type: ContentType::Text,
                body: "Method not allowed".to_string(),
            },
        }
    }

    fn handle_post(&self) -> Response {
        todo!()
    }

    fn handle_get(&self) -> Response {
        if self.path.as_str() == "/" || self.path.as_str() == "/index.html" {
            return Response {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: ContentType::Html,
                body: "".to_string(),
            };
        } else if self.path.starts_with("/echo/") {
            let data = self.path.split_at(6).1;
            Response {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: ContentType::Text,
                body: data.to_string(),
            }
        } else if self.path == "/user-agent" {
            Response {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: ContentType::Text,
                body: self.user_agent.to_owned(),
            }
        } else if self.path.starts_with("/files/") {
            let file_name = self.path.replace("/files", "");
            let env_args: Vec<String> = env::args().collect();
            let mut dir = if env_args.len() < 3 {
                std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            } else {
                env_args[2].clone()
            };
            dir.push_str(&file_name);
            let file_result = fs::read(&dir);
            match file_result {
                Ok(file) => Response {
                    status_code: 200,
                    status_message: "OK".to_string(),
                    content_type: ContentType::OctetStream,
                    body: String::from_utf8(file).expect("file content"),
                },
                Err(_) => Response {
                    status_code: 404,
                    status_message: "Not Found".to_string(),
                    content_type: ContentType::Text,
                    body: "Not Found".to_string(),
                },
            }
        } else {
            Response::not_found()
        }
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("accepted new connection from {}", stream.peer_addr()?);
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer[..])?;
    println!("received data:\n{}", String::from_utf8_lossy(&buffer[..]));

    let request = String::from_utf8(buffer.into())?;
    let lines = request.lines().collect_vec();
    let start_line = lines.get(0).unwrap();
    let split_start_line = start_line.split(' ').collect_vec();
    let _method = split_start_line.get(0).unwrap().to_string();
    let method_type: MethodType = _method.as_str().into();
    let path = split_start_line.get(1).unwrap();
    let _http_version = split_start_line.get(2).unwrap();

    let request: Request = Request::parse(&request).expect("parse request");
    let response = request.handle();
    stream.write(&response.to_bytes())?;

    let host = lines
        .iter()
        .find(|line| line.starts_with("Host: "))
        .unwrap()
        .split(' ')
        .collect_vec()
        .get(1)
        .unwrap();
    Ok(())
}

use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(move || {
                    handle_client(_stream).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
