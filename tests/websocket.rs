use std::borrow::Cow;
use std::env::current_dir;
use std::fs::{metadata, remove_file};
use std::net::TcpStream;
use std::sync::{Mutex, Once};
use std::time::{UNIX_EPOCH, SystemTime};

use actix_web::web::Data;
use cs::file::ProvideFile;
use lazy_static::lazy_static;
use tungstenite::protocol::CloseFrame;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

use cs::api::api::Client;
use cs::data::DataService;
use cs::ws::start_websocket_server;
use cs::ws::ws_message::{AuthRes, CopyRes, Directory, TreeRes};

static PORT: i32 = 9004;

struct FileServiceMock {}

impl ProvideFile for FileServiceMock {
    fn get_tree(&self) -> Result<cs::ws::ws_message::Directory, String> {
        Ok(Directory {
            name: "root".to_string(),
            path: Some("".to_string()),
            dirs: None,
            files: None,
        })
    }
    fn get_file_data(&self, _start: u64, _end: u64, _file_key: String) -> Result<String, String> {
        Ok("data test".to_string())
    }
}

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
    // test with mock FileService
    // static ref FILE_INS: Data<Mutex<FileServiceMock>> = Data::new(Mutex::new(FileServiceMock {}));

    // test with FileService
    static ref FILE_INS: Data<Mutex<cs::file::FileService>> = Data::new(Mutex::new(cs::file::FileService::new(
        "/home/gonzalezext/Pictures/".to_string()
    )));
}

fn before_all() {
    let data_ins = DATA_INS.clone();
    let file_ins = FILE_INS.clone();
    BEFORE_ALL.call_once(|| {
        delete_mock_data();

        // start websocket connection
        start_websocket_server(Data::clone(&data_ins), Data::clone(&file_ins), PORT);
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

fn gen_msg_id() -> i32 {
    i32::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap()
}

fn start_socket_with_auth(
    client_name: String,
    key: String,
    expected_connection_status: bool,
) -> Option<WebSocket<MaybeTlsStream<TcpStream>>> {
    // connect mock client to the websocket server
    let mut socket = loop {
        match connect(format!("ws://localhost:{}/websocket", PORT)) {
            Ok((socket, _)) => break socket,
            Err(_) => continue,
        };
    };

    let id: i32 = gen_msg_id();

    // create authentication message
    let auth_msg = String::from(format!(
        "{{\"id\": {}, \"name\": \"{}\", \"key\":\"{}\", \"type\":\"AuthMsg\"}}",
        id, client_name, key
    ));

    // send auth message to the server
    socket.send(Message::Text(auth_msg)).unwrap();

    // get response from server
    let msg_res = socket.read().expect("Error reading message");
    let auth_res: AuthRes = serde_json::from_str(&*msg_res.to_string()).unwrap();

    // validate response
    assert_eq!(auth_res.id, id);
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

fn get_client_key(client_name: String) -> String {
    let data_service = DATA_INS.clone();
    return data_service
        .lock()
        .unwrap()
        .get_client_key(client_name.clone())
        .unwrap();
}

#[test]
fn ws_auth_test() {
    before_all();
    let data_service = DATA_INS.clone();

    // create dummy data for test
    let client_name = "client_auth_test".to_string();

    // create mock clients
    create_mock_data(vec![client_name.clone()]);

    // get client key
    let key = get_client_key(client_name.clone());

    // start websocket server and get connected with authorization
    let mut socket = start_socket_with_auth(client_name.clone(), key.clone(), true).unwrap();

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
    let client_name = "client2".to_string();

    create_mock_data(vec![client_name.clone()]);

    // start websocket server and get connected with authorization
    start_socket_with_auth(client_name.clone(), "not_valid_key".to_string(), false).unwrap();
}

#[test]
fn ws_get_files_tree_test() {
    before_all();

    // create dummy data for test
    let client_name = "client_get_file_tree".to_string();

    create_mock_data(vec![client_name.clone()]);

    let key = get_client_key(client_name.clone());

    // start websocket server and get connected with authorization
    let mut socket = start_socket_with_auth(client_name.clone(), key, true).unwrap();

    let id: i32 = gen_msg_id();
    let get_tree_msg = String::from(format!("{{\"id\": {},\"type\":\"TreeMsg\"}}", id));

    // send auth message to the server
    socket.send(Message::Text(get_tree_msg)).unwrap();

    // get response from server
    let msg_res = socket.read().expect("Error reading message");
    let tree_res: TreeRes = serde_json::from_str(&*msg_res.to_string()).unwrap();

    print!("{msg_res}");

    assert_eq!(tree_res.id, id);
    assert_eq!(tree_res.root.name.clone(), "root".to_string());
    assert!(false);
}

// #[ignore = "temporaly"]
#[test]
fn ws_copy_file_test() {
    before_all();

    // create dummy data for test
    let client_name = "client_copy_test".to_string();

    create_mock_data(vec![client_name.clone()]);

    let key = get_client_key(client_name.clone());

    // start websocket server and get connected with authorization
    let mut socket = start_socket_with_auth(client_name.clone(), key, true).unwrap();

    // send the copy request to the server
    let id: i32 = gen_msg_id();
    let start = 0;
    let end = 123;
    let file_hash = "9a789e7939e211df6a00022c9d5f4c1f387d8596caf51d8fbaa76c9d96ceb05c".to_string();
    let copy_msg = String::from(format!("{{\"type\":\"CopyMsg\", \"id\": {id}, \"start\": {start}, \"end\": {end}, \"file_hash\": \"{file_hash}\"}}",));
    println!("{copy_msg}");

    // send auth message to the server
    socket.send(Message::Text(copy_msg)).unwrap();

    // get response from server
    let msg_res = socket.read().expect("Error reading message");
    let copy_res: CopyRes = serde_json::from_str(&*msg_res.to_string()).unwrap();

    println!("{:?}", copy_res);

    // validate CopyRes information
    assert_eq!(copy_res.id, id);
    assert_eq!(copy_res.data, "data test".to_string());
    assert_eq!(copy_res.id, id);
    assert_eq!(copy_res.start, start);
    assert_eq!(copy_res.end, end);
}
