// Uncomment this block to pass the first stage
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

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
                let req = HttpRequest::from_incoming_stream(&mut stream);
                match req.path.as_str() {
                    _ if req.path.starts_with("/echo/") => echo_handler(req.path),
                    "/" => ok_handler(),
                    _ => not_found_handler(),
                }
                .write(&mut stream)
                .unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn echo_handler(path: String) -> HttpResponse {
    let to_echo = path.strip_prefix("/echo/").unwrap();

    let mut headers = HashMap::new();
    headers.insert("Content-Length".to_owned(), format!("{}", to_echo.len()));
    headers.insert("Content-Type".to_owned(), "text/plain".to_owned());
    HttpResponse {
        version: "HTTP/1.1".into(),
        response_code: "200 OK".into(),
        headers,
        body: to_echo.into(),
    }
}

fn ok_handler() -> HttpResponse {
    let mut headers = HashMap::new();
    headers.insert("Content-Length".to_owned(), format!("{}", 0));
    headers.insert("Content-Type".to_owned(), "text/plain".to_owned());
    HttpResponse {
        version: "HTTP/1.1".into(),
        response_code: "200 OK".into(),
        headers,
        body: "".into(),
    }
}

fn not_found_handler() -> HttpResponse {
    let mut headers = HashMap::new();
    headers.insert("Content-Length".to_owned(), format!("{}", 0));
    headers.insert("Content-Type".to_owned(), "text/plain".to_owned());
    HttpResponse {
        version: "HTTP/1.1".into(),
        response_code: "404 Not Found".into(),
        headers,
        body: "".into(),
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpRequest {
    fn from_incoming_stream(stream: &mut TcpStream) -> Self {
        let mut buf: [u8; 2048] = [0; 2048];
        let n = stream.read(&mut buf).unwrap();
        let req = str::from_utf8(&buf[..n]).unwrap();

        let mut lines = req.split("\r\n");

        let mut req_line = lines.next().unwrap().split(" ");
        let method = req_line.next().unwrap();
        let path = req_line.next().unwrap();
        let version = req_line.next().unwrap();

        let headers: HashMap<String, String> = HashMap::new();
        while let Some(_header) = lines.next() {
            // TODO: build headers
        }

        let body = ""; // TODO: read body
        Self {
            method: method.to_owned(),
            path: path.to_owned(),
            version: version.to_owned(),
            headers,
            body: body.to_owned(),
        }
    }
}

struct HttpResponse {
    version: String,
    response_code: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    fn write(self, stream: &mut TcpStream) -> Result<()> {
        let mut response = format!("{} {}\r\n", self.version, self.response_code);
        for (k, v) in self.headers {
            response += format!("{}: {}\r\n", k, v).as_str();
        }
        response += format!("\r\n{}", self.body).as_str();
        stream.write(response.as_bytes())?;

        Ok(())
    }
}
