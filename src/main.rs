use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const INDEX_HTML: &str = include_str!("../index.html");

fn main() {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let listener = TcpListener::bind(("0.0.0.0", port)).expect("no se pudo abrir el puerto");
    println!("Servidor escuchando en http://0.0.0.0:{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || handle(stream));
            }
            Err(e) => eprintln!("conexion rechazada: {e}"),
        }
    }
}

fn handle(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().expect("no se pudo clonar el socket"));

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");
    let path = target.split('?').next().unwrap_or("/");

    let mut header = String::new();
    loop {
        header.clear();
        match reader.read_line(&mut header) {
            Ok(0) | Err(_) => break,
            Ok(_) if header == "\r\n" || header == "\n" => break,
            Ok(_) => {}
        }
    }

    let response = match (method, path) {
        ("GET", "/") | ("GET", "/index.html") => {
            respond("200 OK", "text/html; charset=utf-8", INDEX_HTML)
        }
        ("GET", "/health") => respond("200 OK", "text/plain; charset=utf-8", "OK"),
        _ => respond("404 Not Found", "text/html; charset=utf-8", not_found_html()),
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("error al responder: {e}");
    }
    let _ = stream.flush();
}

fn respond(status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    )
}

fn not_found_html() -> &'static str {
    "<!DOCTYPE html><html lang=\"es\"><head><meta charset=\"utf-8\">\
     <title>404</title></head><body><h1>404</h1>\
     <p><a href=\"/\">Volver al inicio</a></p></body></html>"
}