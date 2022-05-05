extern crate core;

use fsio::file;
use fsio::path::as_path::AsPath;
use libwebs::http_magic::{
    HttpHeaders, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode, HttpVersion,
};
use libwebs::{http_magic, utils};
use std::fmt::format;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;
use std::{env, fs, io, net, thread};
use libwebs::config::*;


fn setup() -> TcpListener {
    let tcp_listener = net::TcpListener::bind(BIND_ADDRESS).unwrap_or_else(|err| {
        println!("Could not start server");
        std::process::exit(-1);
    });
    println!("Started server successfully on {}", BIND_ADDRESS);
    tcp_listener
}

// fn sanitize_non_utf8(input: &str) -> String {
//     let mut clean_utf8 = String::with_capacity(input.len());
//     for c in input.as_bytes() {
//         print!("{}",c);
//         // if *c != 0 {
//         //     clean_utf8.push(*c as char)
//         // }
//     }
//     clean_utf8
// }
fn read_stream(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = vec![0u8; 1024];
    let mut data: Vec<u8> = Vec::with_capacity(1024);
    stream
        .set_nonblocking(true)
        .expect("Could not set socket nonblocking");
    loop {
        println!("{}",data.len());
        println!("{}",String::from_utf8_lossy(buffer.as_slice()));
        match stream.read(&mut buffer) {
            Ok(_) => data.extend(&buffer),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
            Err(_) => {
                println!("over here");
                break;
            }
        }
    }
    data
}

fn process_stream(mut stream: TcpStream) {
    let mut request = HttpRequest::default();
    let mut data = read_stream(&mut stream);
    loop {
        let read_request = HttpRequest::from_vec(data.as_slice());
        if read_request.is_none() {
            // so headers not complete yet
            let d = read_stream(&mut stream);
            data.extend(d);
        } else if read_request.as_ref().unwrap().is_body_complete_or_absent() {
            request = read_request.unwrap().clone();
            break;
        } else {
            request = read_request.unwrap().clone();
            while !request.is_body_complete_or_absent() {
                let d = read_stream(&mut stream);
                request.body.extend(d);
            }
            break;
        }
    }
    handle_request(&mut stream, &request);
}

fn handle_request(stream: &mut TcpStream, request: &HttpRequest) {
    let response = if matches!(request.method, HttpMethod::GET) {
        let path = String::from(match request.requested_object.as_ref() {
            "/" | "/index.html" | "/index.htm" => "index.html".to_string(),
            _ => request.requested_object.as_str()[1..].to_string(),
        });
         if path.as_path().exists() {
            let body = fs::read(path.as_path()).unwrap();
            HttpResponse {
                version: request.version.clone(),
                status: HttpStatusCode::Ok,
                headers: HttpHeaders::from([
                    (
                        "Content-Type".to_string(),
                        vec![utils::deduce_file_mime(path.as_path())],
                    ),
                    ("Content-Length".to_string(), vec![body.len().to_string()]),
                ]),
                body,
            }
        } else {
            let body = fs::read(NOT_FOUND).unwrap();
            HttpResponse {
                version: request.version.clone(),
                status: HttpStatusCode::Not_Found,
                headers: HttpHeaders::from([
                    (
                        "Content-Type".to_string(),
                        vec![utils::deduce_file_mime(NOT_FOUND.as_path())],
                    ),
                    ("Content-Length".to_string(), vec![body.len().to_string()]),
                ]),
                body,
            }
        }
    } else if matches!(request.method, HttpMethod::POST) {
        println!("{}",env::current_dir().unwrap().display());
        let path = String::from(&request.requested_object[1..]);
        if path.as_path().exists(){
            let body = fs::read(CONFLICT).unwrap();
            HttpResponse{
                version: request.version.clone(),
                status: HttpStatusCode::Conflict,
                headers: HttpHeaders::from([
                    (
                        "Content-Type".to_string(),
                        vec![utils::deduce_file_mime(CONFLICT.as_path())],
                    ),
                    ("Content-Length".to_string(), vec![body.len().to_string()]),
                ]),
                body
            }
        }else {
            let target_filename = match request.requested_object.as_ref() {
                "/" => utils::random_string(MAX_RAND_FILENAME),
                _ => request.requested_object[1..].to_string().clone()
            };
            println!("{}",target_filename);
            fs::write(target_filename,request.body.as_slice());
            let body = fs::read(CREATED).unwrap();
            HttpResponse{
                version: request.version.clone(),
                status: HttpStatusCode::Created,
                headers: HttpHeaders::from([
                    (
                        "Content-Type".to_string(),
                        vec![utils::deduce_file_mime(CREATED.as_path())],
                    ),
                    ("Content-Length".to_string(), vec![body.len().to_string()]),
                ]),
                body
            }
        }

    }else {
        let body = fs::read(NOT_FOUND).unwrap();
        HttpResponse {
            version: request.version.clone(),
            status: HttpStatusCode::Not_Found,
            headers: HttpHeaders::from([
                (
                    "Content-Type".to_string(),
                    vec![utils::deduce_file_mime(NOT_FOUND.as_path())],
                ),
                ("Content-Length".to_string(), vec![body.len().to_string()]),
            ]),
            body,
        } };
    stream.write_all(response.to_vec().as_slice());

}

fn path_setup() {
    let mut www_root = format!("{}/{}", env::var("HOME").unwrap(), SERVER_ROOT);
    env::set_current_dir(www_root);
}

fn main() {
    path_setup();
    let listener = setup();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build()
        .unwrap();
    for stream in listener.incoming() {
        pool.spawn(move || {
            process_stream(stream.unwrap());
        });
    }
}
