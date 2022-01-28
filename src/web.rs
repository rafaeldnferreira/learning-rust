use std::net::TcpStream;
use std::io::prelude::*;
use std::str;
use crate::financial::{QuoteRetriever, QuoteService};

pub fn handle_connection(mut stream: TcpStream, quote_service: &QuoteService) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // TODO couldn't extract into it's own function due lack of understanding of the borrowing rules
    let path = str::from_utf8(&buffer)
        .unwrap()
        .split(" ")
        .skip(1)
        .take(1)
        .last()
        .expect(" Path is malformed, invalid request");

    const STATUS_OK: &'static str = "HTTP/1.1 200 OK";
    const STATUS_NOT_FOUND: &'static str = "HTTP/1.1 404 NOT FOUND";
    const QUOTES_PATH: &'static str = "/api/v1/quotes";

    let (status_line, contents) = match path {
        x if x == QUOTES_PATH => (STATUS_OK, to_json(quote_service.available_quotes())),
        x if x.starts_with(QUOTES_PATH) &&  x.split("/").count() == 5 => {
            let symbol = x.split("/").last().unwrap();
            if let Some(quote) = quote_service.find(symbol) {
                (STATUS_OK, quote.as_json())
            } else {
                (STATUS_NOT_FOUND, "".to_string())
            }
        }
        _ => (STATUS_NOT_FOUND, "".to_string())
    };

    write_response(&mut stream, status_line, contents);
}

fn write_response(stream: &mut TcpStream, status_line: &str, contents: String) {
    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn to_json(values: Vec<String>) -> String {
    format!("[{}]", values.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<String>>().join(","))
}
