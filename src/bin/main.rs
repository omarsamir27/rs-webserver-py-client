extern crate core;

use libwebs::http_magic;
use libwebs::http_magic::{HttpHeaders, HttpRequest, HttpStatusCode, HttpVersion};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;
use std::{fs, io, net, thread};

fn setup() -> TcpListener {
    let tcp_listener = net::TcpListener::bind("127.0.0.1:80").unwrap_or_else(|err| {
        println!("Could not start server");
        std::process::exit(-1);
    });
    println!("Started server successfully on port 80");
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
    stream.set_nonblocking(true).expect("Could not set socket nonblocking");
    loop {
        match stream.read(&mut buffer) {
            Ok(_) => data.extend(&buffer),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                break
            },
            Err(_) => {
                println!("over here");
                break
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
        if read_request.is_none(){ // so headers not complete yet
            data.extend(read_stream(&mut stream));
        }
        else if read_request.as_ref().unwrap().is_body_complete_or_absent() {
            request = read_request.unwrap().clone();
            break;
        }else{
            request = read_request.unwrap().clone();
            while !request.is_body_complete_or_absent(){
                request.body.extend(read_stream(&mut stream));
            }
            break;
        }
    }
    let file = fs::read("/home/omar/srv/www/Screenshot.png").unwrap();
    let mut hdr = HttpHeaders::new();
    hdr.insert("Content-Type".to_string(), vec!["image/png".to_string()]);
    let lol = http_magic::HttpResponse {
        version: HttpVersion::HTTP1x0,
        status: HttpStatusCode::Ok,
        headers: hdr,
        body: file,
    };

    stream.write_all(lol.to_vec().as_slice());

}

fn handle_request(){

}

fn main() {
    let listener = setup();
    println!("{}",thread::available_parallelism().unwrap());
    println!("{:?}",thread::current().id());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(6).build().unwrap();
    for stream in listener.incoming() {
        pool.spawn(move || {
            println!("{:?}",thread::current().id());
            process_stream(stream.unwrap());
        });

    }

}
