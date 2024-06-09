mod http;

use clap::Parser;
use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    io::Read,
    net::{TcpListener, TcpStream},
    path::Path,
    str,
};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(long, default_value_t = {"/tmp/".to_owned()})]
    directory: String,
}

fn main() {
    let args = Args::parse();

    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let args_copy = args.clone();
                std::thread::spawn(move || handle_connection(stream, args_copy));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, args: Args) {
    let req = HttpRequest::from_incoming_stream(&mut stream);
    match req.path.as_str() {
        _ if req.path.starts_with("/files/") && req.method == "POST" => {
            post_file_handler(req, args)
        }
        _ if req.path.starts_with("/files/") && req.method == "GET" => get_file_handler(req, args),
        _ if req.path.starts_with("/echo/") => echo_handler(req),
        "/user-agent" => echo_user_agent_handler(req),
        "/" => ok_handler(),
        _ => not_found_handler(),
    }
    .write(&mut stream)
    .unwrap();
}

fn post_file_handler(request: HttpRequest, args: Args) -> http::Response {
    let filename = request.path.strip_prefix("/files/").unwrap();
    let path = Path::new(args.directory.as_str()).join(filename);
    fs::write(path, request.body).unwrap();

    http::ResponseBuilder::new()
        .code(http::STATUS_CREATED)
        .build()
}

fn get_file_handler(request: HttpRequest, args: Args) -> http::Response {
    let filename = request.path.strip_prefix("/files/").unwrap();
    let path = Path::new(args.directory.as_str()).join(filename);

    if let Ok(body) = read_to_string(path) {
        http::ResponseBuilder::new()
            .code(http::STATUS_OK)
            .header(http::HEADER_CONTENT_TYPE, "application/octet-stream")
            .body(body)
    } else {
        http::ResponseBuilder::new().code(http::STATUS_NOT_FOUND)
    }
    .build()
}

fn echo_user_agent_handler(request: HttpRequest) -> http::Response {
    http::ResponseBuilder::new()
        .code(http::STATUS_OK)
        .header(http::HEADER_CONTENT_TYPE, "text/plain")
        .body(
            request
                .headers
                .get("User-Agent")
                .unwrap_or(&"".to_owned())
                .clone(),
        )
        .build()
}

fn echo_handler(request: HttpRequest) -> http::Response {
    let to_echo = request.path.strip_prefix("/echo/").unwrap();

    http::ResponseBuilder::new()
        .code(http::STATUS_OK)
        .header(http::HEADER_CONTENT_TYPE, "text/plain")
        .body(to_echo)
        .build()
}

fn ok_handler() -> http::Response {
    http::ResponseBuilder::new().code(http::STATUS_OK).build()
}

fn not_found_handler() -> http::Response {
    http::ResponseBuilder::new()
        .code(http::STATUS_NOT_FOUND)
        .build()
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

        let mut headers: HashMap<String, String> = HashMap::new();
        while let Some(header) = lines.next() {
            if header == "" {
                break;
            }
            let mut header = header.split(": ");
            headers.insert(
                header.next().unwrap().to_owned(),
                header.next().unwrap().to_owned(),
            );
        }

        let body = lines.next().unwrap(); // TODO: read body
        Self {
            method: method.to_owned(),
            path: path.to_owned(),
            version: version.to_owned(),
            headers,
            body: body.to_owned(),
        }
    }
}
