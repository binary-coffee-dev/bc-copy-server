use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::path::Path;
use std::{fs::metadata, fs::File, io::prelude::Write, io::BufReader};

use crate::api::api::Client;

#[derive(Deserialize, Serialize, Clone)]
pub struct Data {
    pub clients: Vec<Client>,
    count: i32,
    version: String,
}

fn get_data_path() -> String {
    // todo: take this path from the application args
    return String::from(format!(
        "{}/data.json",
        current_dir().unwrap().display().to_string()
    ));
}

/** Migrate to db at some point*/
pub fn read_data() -> Data {
    let clients: Vec<Client> = Vec::new();
    let mut data = Data { clients, count: 0, version: "1.0.0".to_string() };
    let path = get_data_path();

    // check that file exist and create it otherwise
    match metadata(&path) {
        Ok(_) => {}
        Err(_) => {
            // create new file
            // Open a file in write-only mode, returns `io::Result<File>`
            let path_obj = Path::new(&path);
            let mut file = match File::create(&path_obj) {
                Err(why) => panic!("couldn't create file {}: {}", path, why),
                Ok(file) => file,
            };

            // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
            match file.write_all(serde_json::to_string(&data).unwrap().as_bytes()) {
                Err(why) => panic!("couldn't write to file {}: {}", path, why),
                Ok(_) => {}
            }
        }
    }

    match File::open(path.clone()) {
        Ok(file) => {
            let reader = BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            data = serde_json::from_reader(reader)
                .expect("Error deserializing list of client from json file.");
        }
        Err(_) => {
            println!("File '{}' couldn't be opened.", path);
        }
    };

    return data;
}

/** Migrate to db at some point */
fn save_data(data: &Data) {
    let path = get_data_path();

    let mut file = File::create(&Path::new(&path)).unwrap();
    match file.write_all(serde_json::to_string(&data).unwrap().as_bytes()) {
        Err(why) => panic!("couldn't write to file {}: {}", path, why),
        Ok(_) => {}
    }
}

pub fn get_client(id: i32) -> Client {
    let data = read_data();

    let index = data
        .clients
        .iter()
        .position(|x| x.id.unwrap() == id)
        .unwrap();
    let client = data.clients.get(index).unwrap().clone();

    return client;
}

pub fn remove_client(id: i32) -> Client {
    let mut data = read_data();

    let index = data
        .clients
        .iter()
        .position(|x| x.id.unwrap() == id)
        .unwrap();
    let removed = data.clients.remove(index);

    save_data(&data);

    return removed;
}

pub fn update_client(client: Client, update_key: bool) -> Client {
    let mut data = read_data();

    let mut list: Vec<Client> = Vec::new();
    for mut it in data.clients.into_iter() {
        if it.id == client.id {
            it.name = client.name.clone();
            if update_key {
                it.key = client.key.clone();
            }
        }
        list.push(it);
    }

    data.clients = list;
    save_data(&data);

    return client;
}

fn gen_str(size: usize) -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect();
}

pub fn gen_key(id: i32) -> Client {
    let mut client = get_client(id);
    
    client.key = Some(format!("{}-{}", gen_str(10), gen_str(5)));

    return update_client(client, true);
}

pub fn new_client(client: Client) -> Client {
    let mut data = read_data();

    // let mut rng = rand::thread_rng();
    let mut client_new = client.clone();
    client_new.id = Some(data.count);

    data.clients.push(client_new.clone());

    data.count += 1;
    save_data(&data);

    return gen_key(client_new.id.unwrap());
}
