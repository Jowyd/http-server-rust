use itertools::Itertools;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpListener;

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
                println!("received data: {}", String::from_utf8_lossy(&buffer[..]));

                let request = String::from_utf8(buffer.into())?;
                let lines = request.lines().collect_vec();
                let start_line = lines.get(0).unwrap();
                let split_start_line = start_line.split(' ').collect_vec();
                let _method = split_start_line.get(0).unwrap();
                let path = split_start_line.get(1).unwrap();
                let _http_version = split_start_line.get(2).unwrap();

                if path == &"/" || path == &"/index.html" {
                    _stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                } else if path.starts_with("/echo/") {
                    let (_, data) = path.split_at(6);
                    _stream.write(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", data.len(), data).as_bytes())?;
                } else {
                    _stream.write(b"HTTP/1.1 404 NOTFOUND\r\n\r\n")?;
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
