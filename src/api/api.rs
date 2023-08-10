use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

#[derive(Deserialize, Serialize)]
struct Client {
    name: String,
}

#[get("/api/clients")]
pub async fn client_api() -> impl Responder {
    let list = vec![
        Client{name: String::from("some")},
        Client{name: String::from("some2")}
    ];
    HttpResponse::Ok().body(to_string(&list).unwrap())
}

// #[get("/api/clients")]
// pub fn client_api() -> &'static str {
//     "Hello, world!"
// }
