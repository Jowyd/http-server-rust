use flate2::write::GzEncoder;
use flate2::Compression;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::{default, fs, thread};
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
#[derive(Copy, Clone)]
enum ContentType {
    Text,
    Html,
    Json,
    OctetStream,
}

impl ContentType {
    fn from_extension(extension: &str) -> ContentType {
        match extension {
            "txt" => ContentType::Text,
            "html" => ContentType::Html,
            "json" => ContentType::Json,
            "bin" => ContentType::OctetStream,
            _ => ContentType::Text,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            ContentType::Text => "text/plain",
            ContentType::Html => "text/html",
            ContentType::Json => "application/json",
            ContentType::OctetStream => "application/octet-stream",
        }
    }
}

enum MethodType {
    GET,
    POST,
    PUT,
    DELETE,
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
    let content_type = response.content_type.to_str();
    let content_length = response.body.len();
    let accept_encoding: &str = if response.accept_encoding.is_some() {
        match response.accept_encoding.as_ref().unwrap() {
            Encoding::Gzip => "Content-Encoding: gzip\r\n",
        }
    } else {
        ""
    };

    return format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-length: {}\r\n{}\r\n",
        response.status_code,
        response.status_message,
        content_type,
        content_length,
        accept_encoding,
    );
}

struct Response {
    status_code: u16,
    status_message: String,
    content_type: ContentType,
    accept_encoding: Option<Encoding>,
    body: Vec<u8>,
}

impl Response {
    fn to_bytes(&mut self) -> Vec<u8> {
        if self
            .accept_encoding
            .is_some_and(|encoding| encoding == Encoding::Gzip)
        {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&self.body).unwrap();
            self.body = encoder.finish().unwrap();
        }

        // let mut header = format_header_response(&self);

        // if self
        //     .accept_encoding
        //     .is_some_and(|encoding| encoding == Encoding::Gzip)
        // {
        //     header.push_str("Content-Encoding: gzip\r\n");
        // }

        // let mut response = header.into_bytes();
        // response.extend_from_slice(&body_bytes);
        // response
        let header = format_header_response(&self);
        let mut result: Vec<u8> = format!("{}", header).as_bytes().to_vec();
        result.extend_from_slice(&self.body);
        return result;
    }

    fn not_found() -> Response {
        Response {
            status_code: 404,
            status_message: "Not Found".to_string(),
            content_type: ContentType::Text,
            accept_encoding: None,
            body: "Not Found".to_string().as_bytes().to_vec(),
        }
    }
}

#[allow(dead_code)]
struct Request {
    method: MethodType,
    path: String,
    http_version: String,
    host: String,
    user_agent: String,
    accept: String,
    accept_encoding: Option<Encoding>,
    body: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq)]
enum Encoding {
    Gzip,
}

impl Encoding {
    fn parse(s: &str) -> Option<Encoding> {
        for part in s.to_lowercase().trim().split(",") {
            let encode = part.trim();
            let encoding = Encoding::parse_single(encode);
            if encoding.is_some() {
                return encoding;
            }
        }
        None
    }

    fn parse_single(s: &str) -> Option<Encoding> {
        match s.to_lowercase().as_str() {
            "gzip" => Some(Encoding::Gzip),
            _ => None,
        }
    }
}

fn get_path() -> String {
    let env_args: Vec<String> = env::args().collect();
    if env_args.len() < 3 {
        return std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    } else {
        return env_args[2].clone();
    }
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
        let mut accept_encoding: Option<Encoding> = None;

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
                    "accept-encoding" => accept_encoding = Encoding::parse(header_value),
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
            accept_encoding,
            body: body.as_bytes().to_vec(),
        })
    }

    fn handle(&self) -> Vec<u8> {
        match self.method {
            MethodType::GET => self.handle_get(),
            MethodType::POST => self.handle_post(),
            _ => Response {
                status_code: 405,
                status_message: "Method Not Allowed".to_string(),
                content_type: ContentType::Text,
                body: "Method not allowed".as_bytes().to_vec(),
                accept_encoding: None,
            }
            .to_bytes(),
        }
    }

    fn handle_post(&self) -> Vec<u8> {
        if self.path.contains("/files/") {
            let file_name = self.path.replace("/files/", "");
            let mut dir = get_path();
            dir.push_str(&file_name);
            let file_creation_result =
                fs::write(dir, String::from_utf8_lossy(&self.body).to_string().trim());
            match file_creation_result {
                Ok(()) => {
                    println!("created");
                    Response {
                        status_code: 201,
                        status_message: "Created".to_string(),
                        content_type: ContentType::Text,
                        body: "".as_bytes().to_vec(),
                        accept_encoding: self.accept_encoding,
                    }
                    .to_bytes()
                }
                Err(_) => Response {
                    status_code: 500,
                    status_message: "Creation Error".to_string(),
                    content_type: ContentType::Text,
                    body: "Error while creating the file".as_bytes().to_vec(),
                    accept_encoding: None,
                }
                .to_bytes(),
            }
        } else {
            Response::not_found().to_bytes()
        }
    }

    fn handle_get(&self) -> Vec<u8> {
        if self.path.as_str() == "/" || self.path.as_str() == "/index.html" {
            return Response {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: ContentType::Html,
                body: "".as_bytes().to_vec(),
                accept_encoding: self.accept_encoding,
            }
            .to_bytes();
        } else if self.path.starts_with("/echo/") {
            let data = self.path.split_at(6).1;
            Response {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: ContentType::Text,
                body: data.as_bytes().to_vec(),
                accept_encoding: self.accept_encoding,
            }
            .to_bytes()
        } else if self.path == "/user-agent" {
            Response {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: ContentType::Text,
                body: self.user_agent.as_bytes().to_owned(),
                accept_encoding: self.accept_encoding,
            }
            .to_bytes()
        } else if self.path.starts_with("/files/") {
            let file_name = self.path.replace("/files", "");
            let mut dir = get_path();
            dir.push_str(&file_name);
            let file_result = fs::read(&dir);
            match file_result {
                Ok(file) => Response {
                    status_code: 200,
                    status_message: "OK".to_string(),
                    content_type: ContentType::OctetStream,
                    body: file,
                    accept_encoding: self.accept_encoding,
                }
                .to_bytes(),
                Err(_) => Response {
                    status_code: 404,
                    status_message: "Not Found".to_string(),
                    content_type: ContentType::Text,
                    body: "Not Found".as_bytes().to_vec(),
                    accept_encoding: None,
                }
                .to_bytes(),
            }
        } else {
            Response::not_found().to_bytes()
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

    let request_str = String::from_utf8(buffer.into())?;

    let request: Request = Request::parse(&request_str).expect("parse request");
    stream.write(&request.handle())?;
    stream.flush()?;
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
