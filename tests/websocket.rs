use std::borrow::Cow;
use std::env::current_dir;
use std::fs::{create_dir, metadata, remove_dir_all, remove_file, File};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Mutex, Once};

use actix_web::web::Data;
use base64::{engine::general_purpose, Engine as _};
use cs::file::{ProvideFile, ReadedData};
use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
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
    fn get_file_data(
        &self,
        _start: u64,
        _end: u64,
        _file_key: String,
    ) -> Result<ReadedData, String> {
        Ok(ReadedData {
            data: "data test".to_string(),
            end: 10,
            last_data: true,
        })
    }
}

fn current_dir_path() -> String {
    current_dir().unwrap().display().to_string()
}

fn get_data_path() -> String {
    String::from(format!("{}/data-test.json", current_dir_path()))
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

fn create_dir_f(dir_path: String) {
    let path_res = format!("{}/{}", current_dir_path(), dir_path).to_string();
    create_dir(path_res).unwrap();
}

fn create_file_f(file_path: String) {
    let path_res = format!("{}/{}", current_dir_path(), file_path).to_string();
    let mut file = File::create(path_res).unwrap();
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(500000)
        .map(char::from)
        .collect();
    file.write_all(rand_string.as_bytes()).unwrap();
}

fn remove_dir_rec(path: String) {
    let path_res = format!("{}/{}", current_dir_path(), path).to_string();
    if remove_dir_all(path_res).is_ok() {}
}

static BEFORE_ALL: Once = Once::new();

lazy_static! {
    static ref DATA_INS: Data<Mutex<DataService>> =
        Data::new(Mutex::new(DataService::new(Some(get_data_path()))));
    // test with mock FileService
    // static ref FILE_INS: Data<Mutex<FileServiceMock>> = Data::new(Mutex::new(FileServiceMock {}));

    // test with FileService
    static ref FILE_INS: Data<Mutex<cs::file::FileService>> = Data::new(Mutex::new(cs::file::FileService::new(
        format!("{}/{}", current_dir_path(), "data-test")
    )));
}

fn before_all() {
    let data_ins = DATA_INS.clone();
    let file_ins = FILE_INS.clone();
    BEFORE_ALL.call_once(|| {
        delete_mock_data();

        // start websocket connection
        start_websocket_server(Data::clone(&data_ins), Data::clone(&file_ins), PORT);

        // testing files
        let data_test_path = "data-test".to_string();
        remove_dir_rec(data_test_path.clone());

        create_dir_f(data_test_path.clone());
        create_file_f(format!("{}/{}", data_test_path.clone(), "A.txt".to_string()).to_string());
        create_file_f(format!("{}/{}", data_test_path.clone(), "B.txt".to_string()).to_string());
        create_file_f(format!("{}/{}", data_test_path.clone(), "C.txt".to_string()).to_string());

        let dir1_path = format!("{}/{}", data_test_path, "dir1").to_string();
        create_dir_f(dir1_path.clone());
        create_file_f(format!("{}/{}", dir1_path.clone(), "A.txt".to_string()).to_string());
        create_file_f(format!("{}/{}", dir1_path.clone(), "B.txt".to_string()).to_string());
        create_file_f(format!("{}/{}", dir1_path.clone(), "C.txt".to_string()).to_string());
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
    rand::thread_rng().gen_range(0..10000000)
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

fn get_tree(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, id: i32) -> TreeRes {
    let get_tree_msg = String::from(format!("{{\"id\": {},\"type\":\"TreeMsg\"}}", id));

    // send auth message to the server
    socket.send(Message::Text(get_tree_msg)).unwrap();

    // get response from server
    let msg_res = socket.read().expect("Error reading message");
    let tree_res: TreeRes = serde_json::from_str(&*msg_res.to_string()).unwrap();

    return tree_res;
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
    let tree_res = get_tree(&mut socket, id);
    // print!("{msg_res}");

    assert_eq!(tree_res.id, id);
    assert_eq!(tree_res.root.name.clone(), "root".to_string());
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

    // request tree from server
    let id: i32 = gen_msg_id();
    let tree_res = get_tree(&mut socket, id);

    let file_name = tree_res.root.files.unwrap().get(0).unwrap().clone();
    let file_path = format!(
        "{}/{}/{}",
        current_dir_path(),
        "data-test",
        file_name.name.clone()
    );
    let file = File::open(file_path.clone()).unwrap();
    let read_size = 300000;
    let mut reader = BufReader::with_capacity(read_size, file);

    let mut start = 0;

    loop {
        let buffer = reader.fill_buf().unwrap();
        let readed_size = buffer.len();
        if readed_size == 0 {
            break;
        }

        // send the copy request to the server
        let id: i32 = gen_msg_id();
        let end = start + read_size;
        let file_hash = file_name.hash.clone();

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
        assert_eq!(copy_res.start, u64::try_from(start).unwrap());
        assert_eq!(copy_res.end, u64::try_from(start + readed_size).unwrap());

        // validate data
        let data_bytes = general_purpose::STANDARD.decode(copy_res.data).unwrap();
        for i in [0..data_bytes.len()] {
            assert_eq!(data_bytes[i.clone()], buffer[i]);
        }

        // free buffer reader
        start += readed_size;
        reader.consume(readed_size);
    }
}
