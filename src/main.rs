use std::{fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

fn main() {
    let listener = TcpListener::bind("localhost:7878").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(error) => println!("Connection Error: {:?}", error),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let tcp_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap_or(String::new()))
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {tcp_request:#?}");

    let request_line = tcp_request.first();

    if request_line.is_none() || !request_line.unwrap().contains("HTTP") {
        println!("Malformed request");
    }

    let request_line = request_line.unwrap();

    if request_line == "GET / HTTP/1.1" {
        let status_line = "HTTP/1.1 200 OK";
        let content = fs::read_to_string("index.html").unwrap();
        let length = content.len();
        
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let content = fs::read_to_string("404.html").unwrap();
        let length = content.len();
        
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

        stream.write_all(response.as_bytes()).unwrap();
    }
}