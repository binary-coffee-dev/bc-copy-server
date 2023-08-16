use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::thread::spawn;

use actix_web::web::Data;
use tungstenite::{accept, Message, WebSocket};

use crate::data::DataService;
use crate::file::{FileService, ProvideFile};
use crate::ws::ws_message::{CopyRes, TreeRes};
use ws_message::{AuthMsg, AuthRes, Message as Msg};

use self::ws_message::{CopyMsg, TreeMsg};

pub mod ws_message;

enum MessageError {
    AuthError(),
}

fn handle_auth_msg(
    msg: AuthMsg,
    data_service: Data<Mutex<DataService>>,
    websocket: &mut WebSocket<TcpStream>,
) -> Result<(), MessageError> {
    println!("AuthMsg: {:?}", msg);
    let data_service = data_service.lock().unwrap();
    let accept = data_service.validate_user_auth(msg.name.clone(), msg.key.clone());

    let res: AuthRes;
    if accept {
        res = AuthRes {
            status: "accepted".to_string(),
        };
    } else {
        res = AuthRes {
            status: "denied".to_string(),
        };
    }
    websocket
        .send(Message::Text(serde_json::to_string(&res).unwrap()))
        .unwrap();
    if !accept {
        return Err(MessageError::AuthError());
    }
    Ok(())
}

fn handle_tree_msg<T: ProvideFile>(
    msg: TreeMsg,
    file_service: Data<Mutex<T>>,
    websocket: &mut WebSocket<TcpStream>,
) -> Result<(), MessageError> {
    println!("TreeMsg: {:?}", msg);

    // setting up tree
    let tree = TreeRes {
        root: file_service.lock().unwrap().get_tree().unwrap(),
    };

    websocket
        .send(Message::Text(serde_json::to_string(&tree).unwrap()))
        .unwrap();

    Ok(())
}

fn handle_copy_msg<T: ProvideFile>(
    msg: CopyMsg,
    file_service: Data<Mutex<T>>,
    websocket: &mut WebSocket<TcpStream>,
) -> Result<(), MessageError> {
    println!("TreeMsg: {:?}", msg);

    let copy_res = CopyRes {
        id: msg.id,
        start: msg.start,
        end: msg.end,
        data: file_service.lock().unwrap().get_file_data(msg.start, msg.end, msg.file_key).unwrap(),
    };

    websocket
        .send(Message::Text(serde_json::to_string(&copy_res).unwrap()))
        .unwrap();

    Ok(())
}

fn user_is_auth(data_service: Data<Mutex<DataService>>, client_name: String) -> bool {
    data_service
        .lock()
        .unwrap()
        .connection_status
        .contains(&client_name)
}

pub fn start_websocket_server<T: ProvideFile + Sync + Send + 'static>(
    data_service_ins: Data<Mutex<DataService>>,
    file_service_ins: Data<Mutex<T>>,
    port: i32,
) {
    let server = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    spawn(move || {
        for stream in server.incoming() {
            let data_service_ins_clone = data_service_ins.clone();
            let file_service_ins_clone = file_service_ins.clone();
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                let mut client_name: Option<String> = None;
                loop {
                    let msg = websocket.read().unwrap();

                    println!("--- {}", msg.to_string());

                    if msg.is_close() {
                        println!("++++++ {:?}", client_name.clone());
                        if client_name.is_some() {
                            data_service_ins_clone
                                .lock()
                                .unwrap()
                                .connection_status
                                .remove(&client_name.clone().unwrap());
                        }
                        break;
                    }

                    // handle message
                    let msg_ins: Msg = serde_json::from_str(&*msg.to_string()).expect("some error");
                    let msg_result = match msg_ins {
                        Msg::AuthMsg(msg) => {
                            client_name = Some(msg.name.clone());
                            handle_auth_msg(msg, data_service_ins_clone.clone(), &mut websocket)
                        }
                        Msg::TreeMsg(msg) => {
                            if !user_is_auth(
                                data_service_ins_clone.clone(),
                                client_name.clone().unwrap(),
                            ) {
                                Err(MessageError::AuthError())
                            } else {
                                handle_tree_msg(msg, file_service_ins_clone.clone(), &mut websocket)
                            }
                        }
                        Msg::CopyMsg(msg) => {
                            if !user_is_auth(
                                data_service_ins_clone.clone(),
                                client_name.clone().unwrap(),
                            ) {
                                Err(MessageError::AuthError())
                            } else {
                                handle_copy_msg(msg, file_service_ins_clone.clone(), &mut websocket)
                            }
                        }
                    };

                    // handle message analisis result
                    match msg_result {
                        Ok(_) => {
                            data_service_ins_clone
                                .lock()
                                .unwrap()
                                .connection_status
                                .insert(client_name.clone().unwrap());
                        }
                        Err(error_type) => {
                            match error_type {
                                MessageError::AuthError() => {
                                    // todo: close connection with websocket
                                }
                            }
                            break;
                        }
                    }
                }
            });
        }
    });
}
