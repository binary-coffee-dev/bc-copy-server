use actix_web::{get, HttpResponse, Responder};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
}

#[get("/")]
pub async fn views() -> impl Responder {
    HttpResponse::Ok().body(Index{}.render().unwrap())
}

