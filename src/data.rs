use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env::{self, current_dir};
use std::sync::Mutex;
use actix_web::web::Data;
use rusqlite::{Connection, params, Statement, ToSql};

use crate::api::api::Client;

#[derive(Deserialize, Serialize, Clone)]
pub struct Configuration {
    version: String,
}

pub struct ConfigurationOption {
    key: String,
    value: String,
}

pub struct DataService {
    pub connection_status: HashSet<String>,
    pub db_connection: Data<Mutex<Connection>>,
}

impl DataService {
    pub fn new() -> DataService {
        let config_path: String = env::var("CONFIG_PATH")
            .unwrap_or(current_dir().unwrap().display().to_string());
        let db_file_name: String = env::var("DB_FILE_NAME")
            .unwrap_or("data.db".to_string());
        println!("{}", db_file_name);
        let config_file = String::from(format!("{}/{}", config_path, db_file_name));
        println!("{}", config_file);

        let db_connection = Connection::open(config_file).unwrap();
        DataService::initialize_db(&db_connection);

        DataService {
            connection_status: HashSet::new(),
            db_connection: Data::new(Mutex::new(db_connection)),
        }
    }

    pub fn initialize_db(db_connection: &Connection) {
        // create initial configuration table
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS configuration ( key TEXT PRIMARY KEY, value TEXT )",
            [],
        ).unwrap();

        // create initial client table
        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS client ( id INTEGER PRIMARY KEY, key TEXT NOT NULL, name TEXT )",
            [],
        ).unwrap();
    }

    /** Migrate to sqlite db at some point*/
    pub fn read_configuration(self: &DataService) -> Configuration {
        let db_connection = self.db_connection.lock().unwrap();
        let mut config = Configuration {
            version: "1.0.0".to_string(),
        };

        let mut stmt = db_connection.prepare(
            "SELECT * FROM configuration;",
        ).unwrap();
        let configs = stmt.query_map([], |row| {
            Ok(ConfigurationOption {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        }).unwrap();

        for conf in configs {
            let it = conf.unwrap();
            if it.key == "version" {
                config.version = it.value;
            }
        }

        return config;
    }

    pub fn get_clients(self: &DataService) -> Vec<Client> {
        let db_connection = self.db_connection.lock().unwrap();
        let mut stmt = db_connection.prepare(
            "SELECT * FROM client;",
        ).unwrap();

        return self.get_client_from_query(&mut stmt, params![]);
    }

    pub fn validate_user_auth(self: &DataService, name: String, key: String) -> bool {
        let db_connection = self.db_connection.lock().unwrap();
        let mut stmt = db_connection.prepare(
            "SELECT * FROM client AS c WHERE c.name = ?1 AND c.key = ?2;",
        ).unwrap();

        let v = self.get_client_from_query(&mut stmt, params![name, key]);
        return !v.is_empty();
    }

    pub fn remove_client(self: &DataService, id: i64) -> Client {
        let client_removed = self.get_client(id);

        let db_connection = self.db_connection.lock();

        db_connection.unwrap().execute(
            "DELETE FROM client AS c WHERE c.id = ?1;",
            [id],
        ).unwrap();

        return client_removed;
    }

    pub fn new_client(self: &DataService, client: Client) -> Client {
        let new_client_id: i64;

        let existing = self.get_client_by_name(client.name.clone().unwrap());
        if existing.is_none() {
            let db_connection = self.db_connection.lock().unwrap();

            db_connection.execute(
                "INSERT INTO client (name, key) VALUES (?1, ?2);",
                [client.name.unwrap(), "".to_string()],
            ).unwrap();
            new_client_id = db_connection.last_insert_rowid();
        } else {
            return existing.unwrap();
        }

        return self.gen_key(new_client_id);
    }

    pub fn get_client_by_name(self: &DataService, client_name: String) -> Option<Client> {
        let db_connection = self.db_connection.lock().unwrap();
        let mut stmt = db_connection.prepare(
            "SELECT * FROM client AS c WHERE c.name = ?1;",
        ).unwrap();

        let v: Vec<Client> = self.get_client_from_query(&mut stmt, params![client_name]);
        return if v.is_empty() { None } else { Some(v[0].clone()) };
    }

    pub fn get_client_key(self: &DataService, client_name: String) -> Option<String> {
        let db_connection = self.db_connection.lock().unwrap();
        let mut stmt = db_connection.prepare(
            "SELECT * FROM client AS c WHERE c.name = ?1;",
        ).unwrap();

        let v: Vec<Client> = self.get_client_from_query(&mut stmt, params![client_name]);
        if v.is_empty() {
            return None;
        } else {
            return Some(v[0].key.clone().unwrap());
        }
    }

    pub fn gen_key(self: &DataService, id: i64) -> Client {
        let mut client = self.get_client(id);

        client.key = Some(format!("{}-{}", self.gen_str(10), self.gen_str(5)));

        return self.update_client(client, true);
    }

    pub fn update_client(self: &DataService, client: Client, update_key: bool) -> Client {
        let current_client = self.get_client(client.id.unwrap());

        {
            let db_connection = self.db_connection.lock().unwrap();

            let key = if update_key { client.key.unwrap() } else { current_client.key.unwrap() };
            db_connection.execute(
                "UPDATE client SET name=?1, key=?2 WHERE id=?3;",
                [client.name.unwrap(), key, format!("{}", client.id.unwrap())],
            ).unwrap();
        }

        return self.get_client(client.id.unwrap());
    }

    pub fn get_client(self: &DataService, id: i64) -> Client {
        let db_connection = self.db_connection.lock().unwrap();
        let mut stmt = db_connection.prepare(
            "SELECT * FROM client AS c WHERE c.id=?1;",
        ).unwrap();

        let clients = self.get_client_from_query(&mut stmt, params![id]);
        return clients[0].clone();
    }

    pub fn get_client_from_query(self: &DataService, stmt: &mut Statement<'_>, params: &[&dyn ToSql]) -> Vec<Client> {
        let clients_mapped = stmt.query_map(params.as_ref(), |row| {
            Ok(Client {
                id: row.get(0)?,
                key: row.get(1)?,
                name: row.get(2)?,
            })
        }).unwrap();

        let mut clients = Vec::new();
        for cli in clients_mapped {
            clients.push(cli.unwrap().clone());
        }

        return clients;
    }

    fn gen_str(self: &DataService, size: usize) -> String {
        return thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size)
            .map(char::from)
            .collect();
    }
}
