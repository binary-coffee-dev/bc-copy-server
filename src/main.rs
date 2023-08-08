#[macro_use]
extern crate rocket;

use rocket_dyn_templates::Template;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

use api::api::client_api;
use api::views::views;

mod api;

#[launch]
fn rocket() -> _ {
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

    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![client_api, views])
}
