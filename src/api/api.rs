use std::sync::Mutex;

use actix_web::{
    delete, get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::data::DataService;

#[derive(Deserialize, Serialize, Clone)]
pub struct Client {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub key: Option<String>,
}

#[get("/api/clients")]
pub async fn client_api_endpoint(data_service_ins: Data<Mutex<DataService>>) -> impl Responder {
    let data_service = data_service_ins.lock().unwrap();
    let clients = data_service.get_clients();
    HttpResponse::Ok().body(to_string(&clients).unwrap())
}

#[post("/api/clients")]
pub async fn create_client_endpoint(
    data_service_ins: Data<Mutex<DataService>>,
    client: web::Form<Client>,
) -> impl Responder {
    let data_service = data_service_ins.lock().unwrap();
    let res = data_service.new_client(client.into_inner());
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[get("/api/clients/{id}")]
pub async fn get_client_endpoint(
    data_service_ins: Data<Mutex<DataService>>,
    id: web::Path<(i64,)>,
) -> impl Responder {
    let data_service = data_service_ins.lock().unwrap();
    let res = data_service.get_client(id.into_inner().0);
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[post("/api/clients/{id}")]
pub async fn update_client_endpoint(
    data_service_ins: Data<Mutex<DataService>>,
    client: web::Form<Client>,
    id: web::Path<(i64,)>,
) -> impl Responder {
    let data_service = data_service_ins.lock().unwrap();
    let mut client_ins = client.into_inner();
    client_ins.id = Some(id.into_inner().0);
    let res = data_service.update_client(client_ins, false);
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[post("/api/clients/{id}/generate_key")]
pub async fn generate_client_key_endpoint(
    data_service_ins: Data<Mutex<DataService>>,
    id: web::Path<(i64,)>,
) -> impl Responder {
    let data_service = data_service_ins.lock().unwrap();
    let res = data_service.gen_key(id.into_inner().0);
    HttpResponse::Ok().body(serde_json::to_string(&res).unwrap())
}

#[delete("/api/clients/{id}")]
pub async fn delete_client_endpoint(
    data_service_ins: Data<Mutex<DataService>>,
    id: web::Path<(i64,)>,
) -> impl Responder {
    let data_service = data_service_ins.lock().unwrap();
    let removed = data_service.remove_client(id.into_inner().0);
    HttpResponse::Ok().body(serde_json::to_string(&removed).unwrap())
}
