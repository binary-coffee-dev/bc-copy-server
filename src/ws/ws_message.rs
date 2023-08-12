use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthMsg {
    pub name: String,
    pub key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthRes {
    pub status: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Message {
    AuthMsg(AuthMsg),
    AuthRes(AuthRes),
}
