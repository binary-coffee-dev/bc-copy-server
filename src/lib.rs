use std::sync::Mutex;
use std::env;

use actix_web::{web::Data};

use data::DataService;
use file::FileService;
use ws::start_websocket_server;
use api::start_api_server;

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
    return start_api_server(webserver_port, data_ins).await;
}
