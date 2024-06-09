use clap::Parser;
use std::{
    collections::HashMap,
    fs::read_to_string,
    io::{Read, Write},
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

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let args_copy = args.clone();
                std::thread::spawn(move || handle(stream, args_copy));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle(mut stream: TcpStream, args: Args) {
    let req = HttpRequest::from_incoming_stream(&mut stream);
    match req.path.as_str() {
        _ if req.path.starts_with("/files/") => file_handler(req, args),
        _ if req.path.starts_with("/echo/") => echo_handler(req),
        "/user-agent" => echo_user_agent_handler(req),
        "/" => ok_handler(),
        _ => not_found_handler(),
    }
    .write(&mut stream)
    .unwrap();
}

fn file_handler(request: HttpRequest, args: Args) -> HttpResponse {
    let filename = request.path.strip_prefix("/files/").unwrap();
    let path = Path::new(args.directory.as_str()).join(filename);

    if let Ok(body) = read_to_string(path) {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_owned(), format!("{}", body.len()));
        headers.insert("Content-Type".to_owned(), "text/octet-stream".to_owned());
        HttpResponse {
            version: "HTTP/1.1".into(),
            response_code: "200 OK".into(),
            headers,
            body,
        }
    } else {
        not_found_handler()
    }
}

fn echo_user_agent_handler(request: HttpRequest) -> HttpResponse {
    HttpResponse::with_body(
        request
            .headers
            .get("User-Agent")
            .unwrap_or(&"".to_owned())
            .clone(),
    )
}

fn echo_handler(request: HttpRequest) -> HttpResponse {
    let to_echo = request.path.strip_prefix("/echo/").unwrap();

    HttpResponse::with_body(to_echo.to_owned())
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
    fn with_body(body: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_owned(), format!("{}", body.len()));
        headers.insert("Content-Type".to_owned(), "text/plain".to_owned());
        Self {
            version: "HTTP/1.1".into(),
            response_code: "200 OK".into(),
            headers,
            body,
        }
    }
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
