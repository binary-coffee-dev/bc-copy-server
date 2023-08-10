use actix_cors::Cors;
use actix_web::{App, HttpServer};

use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

use api::api::client_api;
use api::views::views;

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    spawn(|| {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        for stream in server.incoming() {
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                loop {
                    let msg = websocket.read().unwrap();

                    // We do not want to send back ping/pong messages.
                    if msg.is_binary() || msg.is_text() {
                        websocket.send(msg).unwrap();
                    }
                }
            });
        }
    });

    // Ok(()).await
    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin().allow_any_method().allow_any_header();
        App::new().wrap(cors).service(client_api).service(views)
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
