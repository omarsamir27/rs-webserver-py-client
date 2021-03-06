use crate::http_magic::HttpMethod::GET;
use crate::http_magic::HttpVersion::HTTP1x1;
use crate::utils;
use std::collections::HashMap;
use std::fmt::{write, Display, Formatter};
use std::str::FromStr;

type Result<T> = std::result::Result<T, HttpParseError>;

// #[derive(Copy, Clone)]
pub type HttpHeaders = HashMap<String, Vec<String>>;

pub fn http_headers_fmt(header_map: &HttpHeaders) -> String {
    let mut displayed = String::new();
    for (k, v) in header_map {
        displayed.push_str(k);
        displayed.push_str(": ");
        displayed.push_str(utils::array_stringify(v.as_slice(), ',').as_str());
        displayed.push_str("\r\n");
    }
    displayed.pop();
    displayed.pop();
    displayed
}
#[derive(Debug)]
pub struct HttpParseError {
    detail: String,
}
impl HttpParseError {
    fn new(msg: &str) -> HttpParseError {
        HttpParseError {
            detail: msg.to_string(),
        }
    }
}

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum HttpStatusCode {
    Ok = 200,
    Created = 201,
    Not_Found = 404,
    Conflict = 409,
    Method_Not_Allowed = 405,
}
impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (code, codename) = match self {
            HttpStatusCode::Ok => (200, "OK"),
            HttpStatusCode::Not_Found => (404, "Not Found"),
            HttpStatusCode::Conflict => (409, "Conflict"),
            HttpStatusCode::Created => (201, "Created"),
            HttpStatusCode::Method_Not_Allowed => (405, "Method Not Allowed"),
            _ => (-1, "BAD"),
        };
        write!(f, "{} {}", code, codename)
    }
}

#[derive(Copy, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    HEAD,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    BadMethod,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl HttpMethod {
    fn to_string(&self) -> String {
        let string = match self {
            GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::BadMethod => "BAD METHOD",
        };
        string.to_string()
    }
    pub fn new(method: &str) -> Result<HttpMethod> {
        match method.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::GET),
            "post" => Ok(HttpMethod::POST),
            "head" => Ok(HttpMethod::HEAD),
            "put" => Ok(HttpMethod::PUT),
            "delete" => Ok(HttpMethod::DELETE),
            "connect" => Ok(HttpMethod::CONNECT),
            "options" => Ok(HttpMethod::OPTIONS),
            "trace" => Ok(HttpMethod::TRACE),
            _ => Ok(HttpMethod::BadMethod),
        }
    }
}

#[derive(Clone)]
pub enum HttpVersion {
    HTTP1x0,
    HTTP1x1,
}

impl HttpVersion {
    pub fn new(method: &str) -> Result<HttpVersion> {
        match method {
            "HTTP/1.0" => Ok(HttpVersion::HTTP1x0),
            "HTTP/1.1" => Ok(HttpVersion::HTTP1x1),
            _ => Err(HttpParseError::new("invalid http version")),
        }
    }
}
impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpVersion::HTTP1x0 => {
                write!(f, "HTTP/1.0")
            }
            HttpVersion::HTTP1x1 => {
                write!(f, "HTTP/1.1")
            }
        }
    }
}

pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: HttpStatusCode,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(
        version: HttpVersion,
        status: HttpStatusCode,
        headers: HttpHeaders,
        body: &[u8],
    ) -> HttpResponse {
        HttpResponse {
            version,
            status,
            headers,
            body: body.to_vec(),
        }
    }
    pub fn to_vec(&self) -> Vec<u8> {
        let mut response: Vec<u8> = Vec::new();
        response.extend_from_slice(self.version.to_string().as_bytes());
        response.push(' ' as u8);
        response.extend_from_slice(self.status.to_string().as_bytes());
        response.extend_from_slice("\r\n".as_bytes());
        response.extend_from_slice(http_headers_fmt(&self.headers).as_bytes());
        response.extend_from_slice("\r\n\r\n".as_bytes());
        response.extend_from_slice(self.body.as_slice());
        response
    }
}
impl Display for HttpResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}\r\n{}\r\n\r\n{}",
            self.version.to_string(),
            self.status.to_string(),
            http_headers_fmt(&self.headers),
            String::from_utf8_lossy(&self.body),
        )
    }
}

#[derive(Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub requested_object: String,
    pub version: HttpVersion,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

impl Default for HttpRequest {
    fn default() -> Self {
        HttpRequest {
            method: GET,
            requested_object: String::default(),
            version: HTTP1x1,
            headers: HttpHeaders::default(),
            body: Vec::default(),
        }
    }
}
impl HttpRequest {
    pub fn print_nobody(&self) {
        println!(
            "{} {} {}\r\n{}",
            self.method,
            self.requested_object,
            self.version,
            http_headers_fmt(&self.headers)
        )
    }
    pub fn is_body_complete_or_absent(&self) -> bool {
        let body_len = match self.headers.get("Content-Length") {
            None => return true,
            Some(vec) => usize::from_str(vec[0].as_str().trim()).unwrap(),
        };
        self.body.len() >= body_len
    }
    pub fn headers_terminated(msg: &[u8]) -> Option<usize> {
        let end_index = unsafe {
            msg.iter().position(|x| {
                let x_ptr: *const u8 = x;
                *x_ptr == 13
                    && *(x_ptr.offset(1)) == 10
                    && *(x_ptr.offset(2)) == 13
                    && *(x_ptr.offset(3)) == 10
            })
        };
        end_index
    }
    fn split_body_from_msg(req: &[u8]) -> Option<(String, Vec<u8>)> {
        let has_headers = HttpRequest::headers_terminated(req);
        let body_sep_index = match has_headers {
            None => return None,
            Some(index) => index,
        };

        let rest = &req[..body_sep_index];
        let body = &req[body_sep_index + 4..];
        Some((String::from_utf8_lossy(rest).to_string(), body.to_vec()))
    }
    pub fn from_vec(req: &[u8]) -> Option<HttpRequest> {
        let (rest, mut body) = match HttpRequest::split_body_from_msg(req) {
            None => return None,
            Some(x) => (x.0, x.1),
        };
        let (req_line, headers) = rest.split_once("\r\n").unwrap();
        let req_line: Vec<&str> = req_line.split_ascii_whitespace().collect();
        let (method, requested_object, version) = (
            HttpMethod::new(req_line[0]).unwrap(),
            req_line[1],
            HttpVersion::new(req_line[2]).unwrap(),
        );
        let buff: Vec<String> = headers.lines().map(|line| line.to_string()).collect();
        let mut headers: HashMap<String, Vec<String>> = HashMap::new();
        for line in buff {
            let (header_key, header_values) = line.split_once(':').unwrap();
            let header_values: Vec<String> = header_values
                .split(',')
                .map(|token| token.to_string())
                .collect();
            headers.insert(header_key.to_string(), header_values);
        }
        if headers.get("Content-Length").is_some() {
            let body_length =
                usize::from_str(headers.get("Content-Length").unwrap()[0].as_str().trim()).unwrap();
            body.resize(body_length, 0);
        }
        Some(HttpRequest {
            method,
            requested_object: requested_object.to_string(),
            version,
            headers,
            body,
        })
    }
}
