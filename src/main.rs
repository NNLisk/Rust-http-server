use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::fs;

struct Request {
    method: String,
    route: String,
} 

struct Response {
    status: u16,
    content: String,
}

fn parse_request(stream: &TcpStream) -> Request {
    let buf_reader = BufReader::new(stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result|result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    if let Some(request_line) = http_request.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return Request {
                method: parts[0].to_string(),
                route: parts[1].to_string(),
            };
        } 
    } 

    Request {
        method: String::from("GET"),
        route: String::from("/"),
    }
}

fn respond(stream: &mut TcpStream, response: Response) {
    let status_line = match response.status {
        200 => "HTTP/1.1 200 OK",
        404 => "HTTP/1.1 404 NOT FOUND",
        _ => "HTTP/1.1 500 ERROR",
    };
    
    let length = response.content.len();
    let formatted = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{}",
        response.content
    );
    
    stream.write_all(formatted.as_bytes())
        .expect("Failed to write response");
}


fn handle_connection(mut stream: TcpStream) {
    let request = parse_request(&stream);
    let response = router(&request);
    respond(&mut stream, response);
}


fn router(request: &Request) -> Response {
    match request.route.as_str() {
        "/" => Response {
            status: 200,
            content: fs::read_to_string("contents/index.html")
                .expect("failed reading file"),
        },
        _ => Response {
            status: 404,
            content: String::from("<h1> 404 Not Found</h1>"),
        },
    }
}


fn server() {
    let listener = TcpListener::bind("serveraddress")
        .expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => println!("Failed to accept connection {}", e),
        }
    }
} 

fn main() {
    server();
}