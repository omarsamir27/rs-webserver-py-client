extern crate core;
use fragile::Fragile;
use fsio::file;
use fsio::path::as_path::AsPath;
use lazy_static::lazy_static;
use libwebs::config::*;
use libwebs::control::{
    ControlStat, CONTROL_THREAD_SLEEP, KEEP_ALIVE_TIMEOUT, MAX_KEEP_ALIVE_REQUESTS,
};
use libwebs::http_magic::{
    HttpHeaders, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode, HttpVersion,
};
use libwebs::utils;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use std::sync::RwLock;
use std::time::Duration;
use std::{env, fs, io, mem, net};
use stopwatch::Stopwatch;

lazy_static! {
    static ref OPEN_STREAMS: RwLock<u32> = RwLock::new(0);
    static ref CONTROL_STATS: RwLock<ControlStat> = ControlStat::new();
    static ref OPEN_THREADS: RwLock<u32> = RwLock::new(0);
}

fn setup() -> TcpListener {
    let tcp_listener = net::TcpListener::bind(BIND_ADDRESS).unwrap_or_else(|err| {
        println!("Could not start server");
        std::process::exit(-1);
    });
    println!("Started server successfully on {}", BIND_ADDRESS);
    tcp_listener
}

// fn read_stream(stream: &mut TcpStream) -> Vec<u8> {
//     let mut buffer = vec![0u8; 1024];
//     let mut data: Vec<u8> = Vec::with_capacity(4*1024);
//     stream
//         .set_nonblocking(true)
//         .expect("Could not set socket nonblocking");
//     loop {
//
//         match stream.read_to_end(&mut buffer) {
//             Ok(_) => {
//                 let mut result = 0u64;
//                 for i in 0..buffer.len() {
//                     result+= buffer[i] as u64;
//                 };
//                 if result == 0u64 {
//                     buffer.clear();
//                     break;
//                 }
//                 data.append(&mut buffer);
//                 buffer = vec![0u8;2*data.len()];
//             },
//             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
//             Err(_) => {
//                 println!("over here");
//                 break;
//             }
//         }
//         // let read_size = data.iter().map(|&x| x as u64).sum::<u64>();
//         // let mut result = 0u64;
//         // for i in 0..buffer.len() {
//         //     result+= buffer[i] as u64;
//         // };
//         // if result == 0u64 {
//         //     buffer.clear();
//         //     break;
//         // }
//     }
//     drop(buffer);
//     data
// }

fn read_stream(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    stream.set_read_timeout(Option::from(Duration::from_millis(5)));
    stream.read_to_end(&mut buffer);
    buffer
}

fn process_stream(mut stream: TcpStream) {
    *OPEN_THREADS.write().unwrap() += 1;
    let mut idle_timer = Stopwatch::start_new();
    let mut num_requests = 0u16;
    let thread_index = rayon::current_thread_index().unwrap();
    loop {
        let mut request = HttpRequest::default();
        let mut data = Vec::default();
        loop {
            data.extend(read_stream(&mut stream));
            if data.len() != 0 {
                break;
            } else if idle_timer.elapsed().as_secs() >= KEEP_ALIVE_TIMEOUT as u64
                || num_requests >= MAX_KEEP_ALIVE_REQUESTS
            {
                return;
            } else if *OPEN_STREAMS.read().unwrap() >= *OPEN_THREADS.read().unwrap() {
                if CONTROL_STATS.read().unwrap().thread_index == thread_index {
                    CONTROL_STATS.write().unwrap().reset();
                    return;
                } else if CONTROL_STATS.read().unwrap().idle_time < idle_timer.elapsed() {
                    CONTROL_STATS.write().unwrap().idle_time = idle_timer.elapsed();
                    CONTROL_STATS.write().unwrap().thread_index = thread_index;
                }
            }
        }

        loop {
            let read_request = HttpRequest::from_vec(data.as_slice());
            if read_request.is_none() {
                // so headers not complete yet
                data.extend(read_stream(&mut stream));
            } else if read_request.as_ref().unwrap().is_body_complete_or_absent() {
                request = read_request.unwrap().clone();
                idle_timer.reset();
                break;
            } else {
                request = read_request.unwrap().clone();
                idle_timer.reset();
                while !request.is_body_complete_or_absent() {
                    request.body.extend(read_stream(&mut stream));
                }
                break;
            }
        }
        num_requests += 1;
        handle_request(&mut stream, &request);
        if matches!(request.version, HttpVersion::HTTP1x0)
            || num_requests >= MAX_KEEP_ALIVE_REQUESTS
            || idle_timer.elapsed().as_secs() >= 5
        {
            return;
        }
    }
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
        println!("{}", env::current_dir().unwrap().display());
        let path = String::from(&request.requested_object[1..]);
        if path.as_path().exists() {
            let body = fs::read(CONFLICT).unwrap();
            HttpResponse {
                version: request.version.clone(),
                status: HttpStatusCode::Conflict,
                headers: HttpHeaders::from([
                    (
                        "Content-Type".to_string(),
                        vec![utils::deduce_file_mime(CONFLICT.as_path())],
                    ),
                    ("Content-Length".to_string(), vec![body.len().to_string()]),
                ]),
                body,
            }
        } else {
            let target_filename = match request.requested_object.as_ref() {
                "/" => utils::random_string(MAX_RAND_FILENAME),
                _ => request.requested_object[1..].to_string().clone(),
            };
            println!("{}", target_filename);
            fs::write(target_filename, request.body.as_slice());
            let body = fs::read(CREATED).unwrap();
            HttpResponse {
                version: request.version.clone(),
                status: HttpStatusCode::Created,
                headers: HttpHeaders::from([
                    (
                        "Content-Type".to_string(),
                        vec![utils::deduce_file_mime(CREATED.as_path())],
                    ),
                    ("Content-Length".to_string(), vec![body.len().to_string()]),
                ]),
                body,
            }
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
    };
    stream.set_nonblocking(false);
    stream.write_all(response.to_vec().as_slice());
    stream.flush().unwrap();
}

fn path_setup() {
    let mut www_root = format!("{}/{}", env::var("HOME").unwrap(), SERVER_ROOT);
    env::set_current_dir(www_root);
}

fn main() {
    path_setup();
    let listener = setup();

    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

    for stream in listener.incoming() {
        *OPEN_STREAMS.write().unwrap() += 1;
        *OPEN_THREADS.write().unwrap() = pool.current_num_threads() as u32;
        pool.spawn(|| {
            process_stream(stream.unwrap());
            *OPEN_STREAMS.write().unwrap() -= 1;
        });
    }
}
