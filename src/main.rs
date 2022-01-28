use std::net::TcpListener;
use std::sync::Arc;

use web_server::financial;
use web_server::infra;
use web_server::web;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = infra::ThreadPool::new(4);

    // a shared instance of the quote service
    let quote_service = Arc::new(financial::QuoteService::new());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let quote_service = Arc::clone(&quote_service);
        pool.execute(move || {
            web::handle_connection(stream, &quote_service);
        });
    }

    println!("Shutting down.");
}

