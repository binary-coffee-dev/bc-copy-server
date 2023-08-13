use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::thread::spawn;

use actix_web::web::Data;
use tungstenite::{accept, Message, WebSocket};

use crate::data::DataService;
use ws_message::{AuthMsg, AuthRes, Message as Msg};

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

pub fn start_websocket_server(data_service_ins: Data<Mutex<DataService>>) {
    spawn(move || {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        for stream in server.incoming() {
            let data_service_ins_clone = data_service_ins.clone();
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
