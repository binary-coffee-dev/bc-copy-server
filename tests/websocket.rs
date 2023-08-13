use std::borrow::Cow;
use std::env::current_dir;
use std::fs::remove_file;
use std::sync::Mutex;

use actix_web::web::Data;
use tungstenite::protocol::CloseFrame;
use tungstenite::{connect, Message};

use cs::api::api::Client;
use cs::data::DataService;
use cs::ws::start_websocket_server;
use cs::ws::ws_message::AuthRes;

fn get_data_path() -> String {
    String::from(format!(
        "{}/data-test.json",
        current_dir().unwrap().display().to_string()
    ))
}

fn create_mock_data(clients: Vec<String>) -> DataService {
    let data_service = DataService::new(Some(get_data_path()));

    for name in clients {
        data_service.new_client(Client {
            id: None,
            key: None,
            name: Some(name),
        });
    }

    return data_service;
}

fn delete_mock_data() {
    remove_file(get_data_path()).unwrap();
}

#[test]
fn ws_auth_test() {
    // create dummy data for test
    let port = 9001;
    let client_name = "client1".to_string();

    let data_service = create_mock_data(vec![client_name.clone()]);

    // get client key
    let mut key = "".to_string();
    let data = data_service.read_data();
    for client in data.clients {
        if client.name.unwrap() == client_name {
            key = client.key.unwrap();
            break;
        }
    }

    let data_ins = Data::new(Mutex::new(data_service));

    // todo: set port inside the websocket service too
    // start websocket connection
    start_websocket_server(Data::clone(&data_ins));

    // connect mock client to the websocket server
    let mut socket = loop {
        match connect(format!("ws://localhost:{}/websocket", port)) {
            Ok((socket, _)) => break socket,
            Err(_) => continue,
        };
    };

    // create authentication message
    let msg = String::from(format!(
        "{{\"name\": \"{}\", \"key\":\"{}\", \"type\":\"AuthMsg\"}}",
        client_name, key
    ));

    // send auth message to the server
    socket.send(Message::Text(msg)).unwrap();

    // get response from server
    let msg_res = socket.read().expect("Error reading message");
    let auth_res: AuthRes = serde_json::from_str(&*msg_res.to_string()).unwrap();

    // validate response
    assert_eq!(auth_res.status, "accepted");

    // validate if client is set as connected
    assert!(data_ins
        .lock()
        .unwrap()
        .connection_status
        .contains(&client_name));

    // close websocket connection
    socket
        .close(Some(CloseFrame {
            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: Cow::from("Goodbye"),
        }))
        .unwrap();
    loop {
        if !data_ins
            .lock()
            .unwrap()
            .connection_status
            .contains(&client_name)
        {
            break;
        }
    }
    assert!(!data_ins
        .lock()
        .unwrap()
        .connection_status
        .contains(&client_name));

    // remove data
    delete_mock_data();
}
