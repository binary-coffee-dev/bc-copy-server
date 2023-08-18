use serde::{Deserialize, Serialize};

// AUTH MESSAGE
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthMsg {
    pub id: i32,
    pub name: String,
    pub key: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthRes {
    pub id: i32,
    pub status: String,
}
// AUTH MESSAGE

// TREE MESSAGE
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TreeMsg {
    pub id: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct File {
    pub name: String,
    #[serde(skip_serializing)]
    pub path: Option<String>,
    pub hash: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Directory {
    pub name: String,
    #[serde(skip_serializing)]
    pub path: Option<String>,
    pub dirs: Option<Vec<Directory>>,
    pub files: Option<Vec<File>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TreeRes {
    pub id: i32,
    pub root: Directory,
}
// TREE MESSAGE

// COPY FILE
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CopyMsg {
    pub id: i32,
    pub start: u64,
    pub end: u64,
    pub file_hash: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CopyRes {
    pub id: i32,
    pub start: u64,
    pub end: u64,
    pub data: String,
    pub last_data: bool,
}
// COPY FILE

// ERROR MESSAGE
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ErrRes {
    pub id: i32,
    pub err: String,
}
// ERROR MESSAGE

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Message {
    AuthMsg(AuthMsg),
    TreeMsg(TreeMsg),
    CopyMsg(CopyMsg),
}
