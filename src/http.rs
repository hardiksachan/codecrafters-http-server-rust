use super::Result;
use std::{collections::HashMap, io};

pub const STATUS_OK: &str = "200 OK";
pub const STATUS_NOT_FOUND: &str = "404 Not Found";
pub const STATUS_CREATED: &str = "201 Created";

pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const HEADER_CONTENT_LENGTH: &str = "Content-Length";

#[derive(Debug)]
pub struct Response {
    version: String,
    response_code: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    pub fn write(self, mut stream: impl io::Write) -> Result<()> {
        let mut response = format!("{} {}\r\n", self.version, self.response_code);
        for (k, v) in self.headers {
            response += format!("{}: {}\r\n", k, v).as_str();
        }
        response += format!("\r\n{}", self.body).as_str();
        stream.write(response.as_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MissingResponseCode {}
#[derive(Debug, Default)]
pub struct ResponseCode(String);

#[derive(Debug, Default)]
pub struct ResponseBuilder<C> {
    code: C,
    headers: HashMap<String, String>,
    body: String,
}

impl ResponseBuilder<MissingResponseCode> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C> ResponseBuilder<C> {
    pub fn code(self, code: impl Into<String>) -> ResponseBuilder<ResponseCode> {
        ResponseBuilder {
            code: ResponseCode(code.into()),
            headers: self.headers,
            body: self.body,
        }
    }

    pub fn header(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let mut headers = self.headers;
        headers.insert(name.into(), value.into());
        Self {
            code: self.code,
            body: self.body,
            headers,
        }
    }

    pub fn body(self, body: impl Into<String>) -> Self {
        let body = body.into();
        let res = self.header(HEADER_CONTENT_LENGTH, format!("{}", body.len()));
        Self {
            code: res.code,
            headers: res.headers,
            body,
        }
    }
}

impl ResponseBuilder<ResponseCode> {
    pub fn build(self) -> Response {
        Response {
            version: "HTTP/1.1".into(),
            response_code: self.code.0,
            headers: self.headers,
            body: self.body,
        }
    }
}
