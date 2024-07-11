use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let mut buffer = [0; 1024];
                let _ = _stream.read(&mut buffer[..]).expect("read");
                let request = String::from_utf8_lossy(&buffer[..]);
                println!("request: {}", request);
                if request.contains("GET /index.html HTTP/1.1")
                    || request.contains("GET / HTTP/1.1")
                {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n").expect("200 \n");
                } else {
                    _stream
                        .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        .expect("404 \n");
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
