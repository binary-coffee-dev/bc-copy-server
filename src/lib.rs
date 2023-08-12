use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};

use api::views::views;
use data::DataService;
use ws::start_websocket_server;

pub mod api;
pub mod data;
pub mod ws;

pub async fn run() -> std::io::Result<()> {
    let data = DataService { path: None };
    let data_ins = Data::new(Mutex::new(data));

    start_websocket_server(Data::clone(&data_ins));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .app_data(Data::clone(&data_ins))
            .wrap(cors)
            .service(api::api::client_api_endpoint)
            .service(api::api::get_client_endpoint)
            .service(api::api::generate_client_key_endpoint)
            .service(api::api::update_client_endpoint)
            .service(api::api::delete_client_endpoint)
            .service(api::api::create_client_endpoint)
            .service(views)
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
