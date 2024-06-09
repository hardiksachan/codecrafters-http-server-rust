// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::TcpListener,
    str,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf: [u8; 2048] = [0; 2048];
                let n = stream.read(&mut buf).unwrap();
                let req = str::from_utf8(&buf[..n]).unwrap();
                let mut lines = req.split("\r\n");
                let mut req_line = lines.next().unwrap().split(" ");
                let _method = req_line.next().unwrap();
                let path = req_line.next().unwrap();
                println!("{}", path);
                match path {
                    "/" => stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap(),
                    _ => stream
                        .write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())
                        .unwrap(),
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
