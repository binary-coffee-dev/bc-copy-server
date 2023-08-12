use std::net::TcpListener;
use std::sync::Mutex;
use std::thread::spawn;

use actix_web::web::Data;
use tungstenite::accept;

use crate::data::DataService;
use ws_message::{AuthRes, Message};

pub mod ws_message;

pub fn start_websocket_server(data_service_ins: Data<Mutex<DataService>>) {
    spawn(move || {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        for stream in server.incoming() {
            let data_service_ins_clone = data_service_ins.clone();
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                loop {
                    let msg = websocket.read().unwrap();

                    // let data_service = data_service_ins_clone.lock().unwrap();
                    // We do not want to send back ping/pong messages.
                    println!("{}", msg.to_string());

                    let msg_ins: Message =
                        serde_json::from_str(&*msg.to_string()).expect("some error");
                    match msg_ins {
                        Message::AuthMsg(msg) => {
                            println!("AuthMsg: {:?}", msg);
                            let data_service = data_service_ins_clone.lock().unwrap();
                            let data = data_service.read_data();
                            let mut accept = false;
                            for client in data.clients {
                                if client.name.unwrap() == msg.name
                                    && client.key.unwrap() == msg.key
                                {
                                    accept = true;
                                }
                            }
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
                                .send(tungstenite::Message::Text(
                                    serde_json::to_string(&res).unwrap(),
                                ))
                                .unwrap();
                            if !accept {
                                // close connection
                                // websocket.;
                                break;
                            }
                        }
                        Message::AuthRes(msg) => {
                            println!("AuthMsg: {:?}", msg);
                        }
                    }

                    // if msg.is_binary() || msg.is_text() {
                    //     websocket.send(msg).unwrap();
                    // }
                }
            });
        }
    });
}
