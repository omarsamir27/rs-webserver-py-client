extern crate core;
use fragile::Fragile;
use fsio::file;
use fsio::path::as_path::AsPath;
use libwebs::config::*;
use libwebs::control::{ThreadStats, CONTROL_THREAD_SLEEP, ControlStat};
use libwebs::http_magic::{
    HttpHeaders, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode, HttpVersion,
};
use libwebs::{http_magic, utils};
use num_cpus;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::fmt::format;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::ptr::addr_of_mut;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::{env, fs, io, net, thread};
use std::collections::HashMap;
use rayon::current_thread_index;
use stopwatch::Stopwatch;

// static conf : Vec<i32> = Vec::default();

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
        // println!("{}",data.len());
        // println!("{}",String::from_utf8_lossy(buffer.as_slice()));
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

unsafe fn process_stream(mut stream: TcpStream, control_stats: Arc<HashMap<usize, RwLock<ThreadStats>>>) {
        let control_stats = control_stats.get(&current_thread_index().unwrap()).unwrap();
    loop {
        println!("{:?}", rayon::current_thread_index().unwrap());
        let mut request = HttpRequest::default();
        let mut data = Vec::default();
        loop {
            data.extend(read_stream(&mut stream));
            if data.len() != 0 {
                control_stats.write().unwrap().restart_timer();
                control_stats.write().unwrap().set_processing(true);
                break;
            }
        }

        loop {
            let read_request = HttpRequest::from_vec(data.as_slice());
            if read_request.is_none() {
                // so headers not complete yet
                // let d = read_stream(&mut stream);
                data.extend(read_stream(&mut stream));
            } else if read_request.as_ref().unwrap().is_body_complete_or_absent() {
                request = read_request.unwrap().clone();
                break;
            } else {
                request = read_request.unwrap().clone();
                while !request.is_body_complete_or_absent() {
                    request.body.extend(read_stream(&mut stream));
                }
                break;
            }
        }
        control_stats.write().unwrap().increment_requests();
        handle_request(&mut stream, &request);
        control_stats.write().unwrap().set_processing(false);
        if matches!(request.version, HttpVersion::HTTP1x0) || control_stats.read().unwrap().must_die() {
            control_stats.write().unwrap().die();
            break;
        }
        // println!("{}",rayon::current_num_threads());
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
}

fn path_setup() {
    let mut www_root = format!("{}/{}", env::var("HOME").unwrap(), SERVER_ROOT);
    env::set_current_dir(www_root);
}
//
// unsafe fn control_thread(
//     mut controls: Arc<HashMap<usize, RwLock<ThreadStats>>>,
//     open_streams: &Mutex<usize>,
//     num_threads: usize,
// ) {
//     // loop {
//     // then more streams opened than threads available , must kill some thread
//     println!("controlling");
//     if *open_streams.lock().unwrap() > num_threads {
//         println!("controlling");
//         controls
//             .iter()
//             .max_by(|&x, &y| x.1.read().unwrap().time_since_idle().cmp(&y.1.read().unwrap().time_since_idle()))
//             .unwrap().1.write().unwrap().verdict.store(true,Relaxed);
//     }
//     // }
//     todo!()
// }

fn main() {
    path_setup();
    let listener = setup();
    let num_threads = num_cpus::get_physical() - 1;
    println!("{}", num_threads);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();
    let control = Arc::new(ControlStat::new());
    // let mut stats_vec = HashMap::new();
    // // let mut stats_index = Vec::new();
    // for i in 0..pool.current_num_threads() {
    //     stats_vec.insert(i,RwLock::new(ThreadStats::default()));
    //     // stats_index.push(i);
    // }
    // let share = Arc::new(stats_vec);
    let mut open_streams = Mutex::new(0usize);
    // rayon::spawn(|| control_thread(&mut stats_vec, &open_streams));
    // let mut control_timer = Stopwatch::start_new();
    unsafe {
        for stream in listener.incoming() {
            println!("{:?}", stream.as_ref().unwrap());
            *open_streams.lock().unwrap() += 1;
            pool.scope(|_| {
                let thread_index = rayon::current_thread_index().unwrap();
                process_stream(stream.unwrap(), share.clone());
                *open_streams.lock().unwrap() -= 1;
            });
            if control_timer.elapsed().as_secs() >= CONTROL_THREAD_SLEEP as u64 {
                control_thread(share.clone(), &open_streams, num_threads);
                control_timer.restart();
            }
        }
    }
}
