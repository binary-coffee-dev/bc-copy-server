use std::sync::Mutex;
use std::env;

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};

use api::views::views;
use data::DataService;
use file::FileService;
use ws::start_websocket_server;

pub mod api;
pub mod data;
pub mod file;
pub mod ws;

pub async fn run() -> std::io::Result<()> {
    let data_ins = Data::new(Mutex::new(DataService::new()));
    let data_path: String = env::var("DATA_PATH").unwrap_or("./data".to_string());
    let file_ins = Data::new(Mutex::new(FileService::new(data_path)));

    let websocket_port: String = env::var("WS_PORT").unwrap_or("4001".to_string());
    start_websocket_server(Data::clone(&data_ins), Data::clone(&file_ins), websocket_port.parse().unwrap());

    let webserver_port: String = env::var("WEB_PORT").unwrap_or("4000".to_string());
    println!("WebServer running in port: {}", webserver_port);
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
    .bind(("0.0.0.0", webserver_port.parse::<u16>().unwrap()))?
    .run()
    .await
}
