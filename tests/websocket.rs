use std::borrow::Cow;
use std::env::current_dir;
use std::fs::{metadata, remove_file};
use std::net::TcpStream;
use std::sync::{Mutex, Once};

use actix_web::web::Data;
use lazy_static::lazy_static;
use tungstenite::protocol::CloseFrame;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

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

fn delete_mock_data() {
    let path = get_data_path();
    match metadata(path.clone()) {
        Ok(_) => {
            remove_file(path).unwrap();
        }
        Err(_) => {}
    }
}

static BEFORE_ALL: Once = Once::new();

lazy_static! {
    static ref DATA_INS: Data<Mutex<DataService>> =
        Data::new(Mutex::new(DataService::new(Some(get_data_path()))));
}

fn before_all() {
    let data_ins = DATA_INS.clone();
    BEFORE_ALL.call_once(|| {
        delete_mock_data();

        // todo: set port inside the websocket service too
        // start websocket connection
        start_websocket_server(Data::clone(&data_ins));
    });
}

fn create_mock_data(clients: Vec<String>) {
    for name in clients {
        let data_service = DATA_INS.clone();
        data_service.lock().unwrap().new_client(Client {
            id: None,
            key: None,
            name: Some(name),
        });
    }
}

fn start_socket_with_auth(
    client_name: String,
    key: String,
    port: i32,
    expected_connection_status: bool,
) -> Option<WebSocket<MaybeTlsStream<TcpStream>>> {
    // connect mock client to the websocket server
    let mut socket = loop {
        match connect(format!("ws://localhost:{}/websocket", port)) {
            Ok((socket, _)) => break socket,
            Err(_) => continue,
        };
    };

    // create authentication message
    let auth_msg = String::from(format!(
        "{{\"name\": \"{}\", \"key\":\"{}\", \"type\":\"AuthMsg\"}}",
        client_name, key
    ));

    // send auth message to the server
    socket.send(Message::Text(auth_msg)).unwrap();

    // get response from server
    let msg_res = socket.read().expect("Error reading message");
    let auth_res: AuthRes = serde_json::from_str(&*msg_res.to_string()).unwrap();

    // validate response
    assert_eq!(
        auth_res.status,
        if expected_connection_status {
            "accepted"
        } else {
            "denied"
        }
    );

    return Some(socket);
}

#[test]
fn ws_auth_test() {
    before_all();
    let data_service = DATA_INS.clone();

    // create dummy data for test
    let port = 9001;
    let client_name = "client1".to_string();

    // create mock clients
    create_mock_data(vec![client_name.clone()]);

    // get client key
    let key;
    {
        key = data_service
            .lock()
            .unwrap()
            .get_client_key(client_name.clone())
            .unwrap();
    }

    // start websocket server and get connected with authorization
    let mut socket = start_socket_with_auth(client_name.clone(), key.clone(), port, true).unwrap();

    // close websocket connection
    socket
        .close(Some(CloseFrame {
            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: Cow::from("Goodbye"),
        }))
        .unwrap();
    // wait until the server update the client connection status
    loop {
        if !data_service
            .lock()
            .unwrap()
            .connection_status
            .contains(&client_name)
        {
            break;
        }
    }
    assert!(!data_service
        .lock()
        .unwrap()
        .connection_status
        .contains(&client_name));
}

#[test]
fn ws_auth_with_wrong_password_test() {
    before_all();

    // create dummy data for test
    let port = 9001;
    let client_name = "client2".to_string();

    create_mock_data(vec![client_name.clone()]);

    // start websocket server and get connected with authorization
    start_socket_with_auth(
        client_name.clone(),
        "not_valid_key".to_string(),
        port,
        false,
    )
    .unwrap();
}

