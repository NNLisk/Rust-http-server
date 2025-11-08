use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs;

struct Request {
    method: String,
    route: String,
} 

impl Request {
    fn default() -> Request{
        Request {
            method: String::from("GET"),
            route: String::from("/"),
        }
    }
} 

struct Response {
    status: u16,
    content: Vec<u8>,
}

impl Response {
    fn default_not_found() -> Response {
        Response {
            status: 400,
        content: b"<h1>400 Bad Request </h1>".to_vec(),
        }
    }
}

async fn parse_request(stream: &mut TcpStream) -> Option<Request> {
    let buf_reader = BufReader::new(stream);
    let mut lines = buf_reader.lines();

    let mut http_request = Vec::new();

    while let Ok(Some(line)) = lines.next_line().await {
        if line.is_empty() {
            break;
        }
        http_request.push(line);
    }

    println!("Request: {http_request:#?}");

    if let Some(request_line) = http_request.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(Request {
                method: parts[0].to_string(),
                route: parts[1].to_string(),
            });
        } 
    } 
    Some(Request::default())
}

async fn respond(stream: &mut TcpStream, response: Response) {
    let status_line = match response.status {
        200 => "HTTP/1.1 200 OK",
        404 => "HTTP/1.1 404 NOT FOUND",
        _ => "HTTP/1.1 500 ERROR",
    };
    
    let length = response.content.len();
    let header = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n",
    );
    
    stream.write_all(header.as_bytes())
        .await
        .expect("Failed to write response header");

    stream.write_all(&response.content)
        .await
        .expect("Failed to write response body")
}


async fn router(request: &Request) -> Response {
    let file_path = if request.route == "/" {
        "portfolio/index.html".to_string()
    } else {
        format!("portfolio{}", request.route)
    };
    
    match fs::read(&file_path).await {
        Ok(content) => Response {
            status: 200,
            content,
        },
        Err(_) => Response {
            status: 404,
            content: b"<h1>404 Not Found</h1>".to_vec(),
        },
    }
}

async fn handle_connection(mut stream: TcpStream) {
    
    match parse_request(&mut stream).await {
        Some(request) => {
            if validate_path(&request) {
                let response = router(&request).await;
                respond(&mut stream, response).await;
            } else {
                let response = Response::default_not_found();
                respond(&mut stream, response).await;
            }
        },
        None => {
            let response = Response::default_not_found();
            respond(&mut stream, response).await;
        }
    }
}

async fn server() {
    let listener = TcpListener::bind("192.168.100.10:7878")
        .await
        .expect("Failed to bind");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("Connection from {}", addr);
                tokio::spawn(async move {
                    handle_connection(stream).await;
                });
            }
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

#[tokio::main]
async fn main() {
    server().await;
}

