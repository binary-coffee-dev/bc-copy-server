use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};

use crate::data::DataService;

pub mod api;
pub mod views;

use views as index;

pub async fn start_api_server(webserver_port: String,
                              data_ins: Data<Mutex<DataService>>,
) -> std::io::Result<()> {
    println!("WebServer running in port: {}", webserver_port);
    return HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .app_data(Data::clone(&data_ins))
            .wrap(cors)
            .service(api::client_api_endpoint)
            .service(api::get_client_endpoint)
            .service(api::generate_client_key_endpoint)
            .service(api::update_client_endpoint)
            .service(api::delete_client_endpoint)
            .service(api::create_client_endpoint)
            .service(index::views)
    })
        .bind(("0.0.0.0", webserver_port.parse::<u16>().unwrap()))?
        .run()
        .await;
}
