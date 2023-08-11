use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::data;

#[derive(Deserialize, Serialize, Clone)]
pub struct Client {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub key: Option<String>,
}

#[get("/api/clients")]
pub async fn client_api_endpoint() -> impl Responder {
    let clients = data::read_clients();
    HttpResponse::Ok().body(to_string(&clients).unwrap())
}

#[post("/api/clients")]
pub async fn create_client_endpoint(client: web::Form<Client>) -> impl Responder {
    let res = data::new_client(client.into_inner());
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[get("/api/clients/{id}")]
pub async fn get_client_endpoint(id: web::Path<(i32,)>) -> impl Responder {
    let res = data::get_client(id.into_inner().0);
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[post("/api/clients/{id}")]
pub async fn update_client_endpoint(
    client: web::Form<Client>,
    id: web::Path<(i32,)>,
) -> impl Responder {
    let mut client_ins = client.into_inner();
    client_ins.id = Some(id.into_inner().0);
    let res = data::update_client(client_ins, false);
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[post("/api/clients/{id}/generate_key")]
pub async fn generate_client_key_endpoint(id: web::Path<(i32,)>) -> impl Responder {
    let res = data::gen_key(id.into_inner().0);
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[delete("/api/clients/{id}")]
pub async fn delete_client_endpoint(id: web::Path<(i32,)>) -> impl Responder {
    let removed = data::remove_client(id.into_inner().0);
    HttpResponse::Ok().body(serde_json::to_string(&removed).unwrap())
}
