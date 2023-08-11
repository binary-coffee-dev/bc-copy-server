use rand::Rng;
use std::env::current_dir;
use std::path::Path;
use std::{fs::metadata, fs::File, io::prelude::Write, io::BufReader};

use crate::api::api::Client;

fn get_data_path() -> String {
    // todo: take this path from the application args
    return String::from(format!(
        "{}/data.json",
        current_dir().unwrap().display().to_string()
    ));
}

/** Migrate to db */
pub fn read_clients() -> Vec<Client> {
    let mut clients: Vec<Client> = Vec::new();
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
            match file.write_all(serde_json::to_string(&clients).unwrap().as_bytes()) {
                Err(why) => panic!("couldn't write to file {}: {}", path, why),
                Ok(_) => {}
            }
        }
    }

    match File::open(path.clone()) {
        Ok(file) => {
            let reader = BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            clients = serde_json::from_reader(reader)
                .expect("Error deserializing list of client from json file.");
        }
        Err(_) => {
            println!("File '{}' couldn't be opened.", path);
        }
    };

    return clients;
}

/** Migrate to db */
fn save_clients(clients: &Vec<Client>) {
    let path = get_data_path();

    let mut file = File::create(&Path::new(&path)).unwrap();
    match file.write_all(serde_json::to_string(&clients).unwrap().as_bytes()) {
        Err(why) => panic!("couldn't write to file {}: {}", path, why),
        Ok(_) => {}
    }
}

pub fn get_client(id: i32) -> Client {
    let clients = read_clients();

    let index = clients.iter().position(|x| x.id.unwrap() == id).unwrap();
    let client = clients.get(index).unwrap().clone();

    return client;
}

pub fn remove_client(id: i32) -> Client {
    let mut clients = read_clients();

    let index = clients.iter().position(|x| x.id.unwrap() == id).unwrap();
    let removed = clients.remove(index);

    save_clients(&clients);

    return removed;
}

pub fn update_client(client: Client, update_key: bool) -> Client {
    let clients = read_clients();

    let mut list: Vec<Client> = Vec::new();
    for mut it in clients.into_iter() {
        if it.id == client.id {
            it.name = client.name.clone();
            if update_key {
                it.key = client.key.clone();
            }
        }
        list.push(it);
    }

    save_clients(&list);

    return client;
}

pub fn gen_key(id: i32) -> Client {
    let mut client = get_client(id);
    client.key = Some("newkey".to_string());

    return update_client(client, true);
}

pub fn new_client(client: Client) -> Client {
    let mut clients = read_clients();

    let mut rng = rand::thread_rng();
    let mut client_new = client.clone();
    client_new.id = Some(rng.gen_range(0..10000));

    clients.push(client_new.clone());

    save_clients(&clients);

    return gen_key(client_new.id.unwrap());
}
