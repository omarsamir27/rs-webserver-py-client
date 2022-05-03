/* BODY IMPLEMENTED AS STRING IN HTTP REQUEST AND RESPONSE BUT IT CAN BE BINARY DATA SO REPAIR THIS*/
/* BODY IMPLEMENTED AS STRING IN HTTP REQUEST AND RESPONSE BUT IT CAN BE BINARY DATA SO REPAIR THIS*/
/* BODY IMPLEMENTED AS STRING IN HTTP REQUEST AND RESPONSE BUT IT CAN BE BINARY DATA SO REPAIR THIS*/
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Write;
use crate::utils;

type Result<T> = std::result::Result<T, HttpParseError>;
type HttpHeaders = HashMap<String, Vec<String>>;

pub fn http_headers_fmt(header_map:&HttpHeaders) -> String{
    let mut displayed = String::new();
    for (k,v) in header_map{
        displayed.push_str(k);
        displayed.push_str(": ");
        displayed.push_str(utils::array_stringify(v.as_slice(),',').as_str());
        displayed.push_str("\r\n");
    };
    displayed
}

#[derive(Debug, Clone)]
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

#[allow(non_camel_case_types)]
pub enum HttpStatusCode {
    Ok = 200,
    Not_Found = 404,
}
impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (code, codename) = match self {
            HttpStatusCode::Ok => (200, "OK"),
            HttpStatusCode::Not_Found => (404, "Not Found"),
            _ => (-1, "BAD"),
        };
        write!(f, "{} {}", code, codename)
    }
}

pub enum HttpMethod {
    GET,
    POST,
}
impl HttpMethod {
    pub fn new(method: &str) -> Result<HttpMethod> {
        match method.to_lowercase().as_str() {
            "get" => Ok(HttpMethod::GET),
            "post" => Ok(HttpMethod::POST),
            _ => Err(HttpParseError::new("invalid http method")),
        }
    }
}

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
    pub fn new(version:HttpVersion,status:HttpStatusCode,headers:HttpHeaders,body:&[u8])-> HttpResponse{
        HttpResponse{version,status,headers,body:body.to_vec()}
    }
}
impl Display for HttpResponse{
     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{} {}\r\n{}\r\n\r\n{}",
            self.version.to_string(),
            self.status.to_string(),
            http_headers_fmt(&self.headers),
            String::from_utf8_lossy(&self.body),
        )
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub requested_object: String,
    pub version: HttpVersion,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}
impl HttpRequest {
    fn split_body_from_msg(req: &[u8]) -> (String, Vec<u8>) {
        let body_sep_index = unsafe {
            req.iter()
                .position(|x| {
                    let x_ptr: *const u8 = x;
                    *x_ptr == 13
                        && *(x_ptr.offset(1)) == 10
                        && *(x_ptr.offset(2)) == 13
                        && *(x_ptr.offset(3)) == 10
                })
                .unwrap()
        };
        let rest = &req[..body_sep_index];
        let body = &req[body_sep_index + 4..];
        (String::from_utf8_lossy(rest).to_string(), body.to_vec())
    }
    pub fn from_vec(req: &[u8]) -> HttpRequest {
        let (rest, body) = HttpRequest::split_body_from_msg(req);
        let (req_line, headers) = rest.split_once("\r\n").unwrap();
        let req_line: Vec<&str> = req_line.split_ascii_whitespace().collect();
        let (method, requested_object, version) = (
            HttpMethod::new(req_line[0]).unwrap(),
            req_line[1],
            HttpVersion::new(req_line[2]).unwrap(),
        );
        // let (headers, body) = rest.split_once("\r\n\r\n").unwrap();
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
        HttpRequest {
            method,
            requested_object: requested_object.to_string(),
            version,
            headers,
            body,
        }
    }
}
