use std::{fs, io::{Error, Write}, net::{TcpListener, TcpStream}, thread, time::Duration};

use http::{HttpMethod, HttpRequest, HttpResponse, HttpStatusCode};
use rust_http_server::ThreadPool;

mod http;

fn main() {
    let host = "localhost";
    let port = 7878;

    let listener = TcpListener::bind(format!("{host}:{port}")).unwrap();
    let pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        pool.execute(|| {
            process_tcp_stream(stream);
        });
    }

    println!("Shutting down.")
}

fn process_tcp_stream(stream: Result<TcpStream, Error>) {
    match stream {
        Ok(stream) => handle_connection(stream),
        Err(error) => println!("Connection Error: {:?}", error),
    }
}

fn handle_connection(mut stream: TcpStream) {
    let response = match HttpRequest::parse(&stream) {
        Ok(request) => handle_http_request(request),
        Err(error) => {
                match error.kind {
                    http::HttpErrorKind::NoData => {
                        return;
                    },
                    http::HttpErrorKind::MalformedRequest => {
                        stream.write_all(error.message.as_bytes()).unwrap();
                        return;
                    },
                }
            }
    };

    let response_str = response.to_string();
    println!("{}", response_str);
    stream.write_all(response_str.as_bytes()).unwrap();
}

fn handle_http_request(request: HttpRequest) -> HttpResponse {
    if request.version != "HTTP/1.1" {
        return HttpResponse::new(HttpStatusCode::HTTPVersionnotsupported, None, None);
    }

    if !matches!(request.method, HttpMethod::GET) {
        return HttpResponse::new(HttpStatusCode::MethodNotAllowed, None, None);
    }

    match request.uri.as_str() {
        "/" => {
            let content = fs::read_to_string("index.html").unwrap();
            HttpResponse::new(HttpStatusCode::OK, None, Some(content))
        },
        "/sleep" => {
            thread::sleep(Duration::from_secs(10));
            let content = fs::read_to_string("index.html").unwrap();
            HttpResponse::new(HttpStatusCode::OK, None, Some(content))
        }
        _ => {
            let content = fs::read_to_string("404.html").unwrap();
            HttpResponse::new(HttpStatusCode::NotFound, None, Some(content))
        },
    }
}