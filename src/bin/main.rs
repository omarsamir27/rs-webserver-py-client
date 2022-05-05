extern crate core;

use libwebs::{http_magic, utils};
use libwebs::http_magic::{HttpHeaders, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode, HttpVersion};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;
use std::{env, fs, io, net, thread};
use std::fmt::format;
use fsio::file;
use fsio::path::as_path::AsPath;

const SERVER_ROOT: &str = "srv/www" ;
const BIND_ADDRESS:&str = "127.0.0.1:1025";

fn setup() -> TcpListener {
    let tcp_listener = net::TcpListener::bind(BIND_ADDRESS).unwrap_or_else(|err| {
        println!("Could not start server");
        std::process::exit(-1);
    });
    println!("Started server successfully on {}",BIND_ADDRESS);
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
    handle_request(&mut stream,&request);

}

fn handle_request(stream:&mut TcpStream,request:&HttpRequest){
    if matches!(request.method,HttpMethod::GET) {
        let mut path= request.requested_object.as_str()[1..].to_string();
        if file::ensure_exists(path.as_str()).is_ok(){
            let content = fs::read(path.as_str()).unwrap();
            let response = HttpResponse{
                version: request.version.clone(),
                status: HttpStatusCode::Ok,
                headers: HttpHeaders::from([
                    ("Content-Type".to_string(),vec![utils::deduce_file_mime(path.as_path())]),
                    ("Content-Length".to_string(),vec![content.len().to_string()])
                ]),
                body:content
            };
            stream.write_all(response.to_vec().as_slice());
        }
    }
}

fn path_setup(){
    let mut www_root = format!("{}/{}",env::var("HOME").unwrap(),SERVER_ROOT);
    println!("{}",www_root);
    env::set_current_dir(www_root);
    println!("{}",env::current_dir().unwrap().display());
}

fn main() {
    path_setup();
    let listener = setup();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(6).build().unwrap();
    for stream in listener.incoming() {
        pool.spawn(move || {
            process_stream(stream.unwrap());
        });

    }

}
