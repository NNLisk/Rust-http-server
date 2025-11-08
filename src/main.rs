use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*};
use std::fs;
use std::thread;
use std::time::Duration;

struct Request {
    method: String,
    route: String,
} 

impl Request {
    fn default() -> Request{
        return Request {
            method: String::from("GET"),
            route: String::from("/"),
        }
    }
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
    Request::default()
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


fn router(request: &Request) -> Response {
    match request.route.as_str() {
        "/" => Response {
            status: 200,
            content: fs::read_to_string("portfolio/index.html")
            .expect("failed reading file"),
        },
        _ => {
            
            let file_path = format!("portfolio{}", request.route);
            println!("portfolio{}", request.route);
            match fs::read_to_string(&file_path) {
                Ok(content) => Response {
                    status: 200,
                    content,
                },
                Err(_) => Response {
                    status: 404,
                    content: String::from("<h1>404 Not Found</h1>"),
                },
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request = parse_request(&stream);
    
    match validate_path(&request) {
        true => {
            let response = router(&request);
            respond(&mut stream, response);
        },
        false => {
            println!("Path validation failed.")
        }
    } 
}

fn server() {
    let listener = TcpListener::bind("192.168.100.10:7878")
        .expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => println!("Failed to accept connection {}", e),
        }
    }
} 

// simple path security
fn validate_path(request: &Request) -> bool {
    if request.route.contains("..") {
        return false;
    }
    true
}

fn main() {
    server();
}

