extern crate core;
use libwebs::http_magic;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;
use std::{fs, net};

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

fn read_stream<'a>(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = vec![0u8; 1024];
    let mut data: Vec<u8> = Vec::with_capacity(1024);
    stream.set_read_timeout(Some(Duration::new(2, 0)));
    while stream.read(&mut buffer).unwrap_or(0) != 0 {
        data.extend(buffer.iter());
    }
    data
}

fn process_stream(mut stream: TcpStream) {
    let data = read_stream(&mut stream);
    let request = http_magic::HttpRequest::from_vec(data.as_slice());

    let lol = "lol ya negm";
    fs::write(Path::new("/home/omar/testout.txt"), req.body);
    stream.write(lol.as_bytes());
}

fn main() {
    let listener = setup();
    for stream in listener.incoming() {
        // println!("{:?}",stream.unwrap());
        process_stream(stream.unwrap())
    }
}
