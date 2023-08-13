use serde::{Deserialize, Serialize};

// AUTH MESSAGE
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthMsg {
    pub name: String,
    pub key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthRes {
    pub status: String,
}
// AUTH MESSAGE

// TREE MESSAGE
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TreeMsg {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct File {
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Directory {
    pub name: String,
    pub dirs: Option<Vec<Directory>>,
    pub files: Option<Vec<File>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TreeRes {
    pub root: Directory,
}
// TREE MESSAGE

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Message {
    AuthMsg(AuthMsg),
    TreeMsg(TreeMsg),
}
