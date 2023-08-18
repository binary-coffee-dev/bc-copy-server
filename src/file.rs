use std::cmp::min;
use std::fs::File as Fl;
use std::os::unix::prelude::FileExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use std::{collections::HashMap, fs::read_dir};

use crate::ws::ws_message::{Directory, File};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sha256::try_digest;

struct PathCash {
    time: Instant,
    path: String,
    size: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ReadedData {
    pub data: String,
    pub end: u64,
    pub last_data: bool,
}

pub trait ProvideFile {
    fn get_tree(&self) -> Result<Directory, String>;
    fn get_file_data(&self, start: u64, end: u64, file_key: String) -> Result<ReadedData, String>;
}

pub struct FileService {
    root_path: String,
    files_hash: Arc<Mutex<HashMap<String, PathCash>>>,
}

impl FileService {
    pub fn new(root_path: String) -> FileService {
        let files_hash = Arc::new(Mutex::new(HashMap::new()));

        let service = FileService {
            root_path,
            files_hash,
        };

        service.start_cash_timeout_checker();

        return service;
    }

    fn start_cash_timeout_checker(self: &FileService) {
        let map = self.files_hash.clone();
        spawn(move || loop {
            let mut to_remove: Vec<String> = Vec::new();
            for (hash, path_cash) in map.lock().unwrap().iter() {
                if path_cash.time.elapsed() > Duration::from_secs(30000) {
                    to_remove.push(hash.clone());
                }
            }
            for r in to_remove {
                map.lock().unwrap().remove(&r);
            }
            sleep(Duration::from_secs(5));
        });
    }

    fn get_tree_rec(self: &FileService, path: &Path, dir: &mut Directory) {
        for entry in read_dir(path).unwrap() {
            let new_path = entry.unwrap().path();
            if new_path.is_dir() {
                let mut new_dir = Directory {
                    name: new_path.file_name().unwrap().to_str().unwrap().to_string(),
                    path: Some(new_path.to_str().unwrap().to_string()),
                    files: Some(Vec::new()),
                    dirs: Some(Vec::new()),
                };
                self.get_tree_rec(&new_path, &mut new_dir);

                let mut dirs = dir.dirs.clone().unwrap();
                dirs.push(new_dir);
                dir.dirs = Some(dirs);
            } else {
                let new_file = File {
                    name: new_path.file_name().unwrap().to_str().unwrap().to_string(),
                    path: Some(new_path.to_str().unwrap().to_string()),
                    hash: try_digest(new_path.clone()).unwrap(),
                    size: new_path.metadata().unwrap().len(),
                };

                let mut files = dir.files.clone().unwrap();
                files.push(new_file);
                dir.files = Some(files);
            }
        }
    }

    fn get_file_list(self: &FileService, path: &Path) -> Vec<File> {
        let mut res = Vec::new();
        let mut q = Vec::new();
        q.push(path.to_str().unwrap().to_string());
        while !q.is_empty() {
            let dir_path: String = q.pop().unwrap();
            for entry in read_dir(dir_path).unwrap() {
                let new_path = entry.unwrap().path();
                if new_path.is_dir() {
                    let path_str = new_path.to_str().unwrap().to_string();
                    q.push(path_str);
                } else {
                    let new_file = File {
                        name: new_path.file_name().unwrap().to_str().unwrap().to_string(),
                        path: Some(new_path.to_str().unwrap().to_string()),
                        hash: try_digest(new_path.clone()).unwrap(),
                        size: new_path.metadata().unwrap().len(),
                    };
                    res.push(new_file);
                }
            }
        }
        return res;
    }

    fn read_data(
        self: &FileService,
        path: String,
        file_len: u64,
        start: u64,
        end: u64,
    ) -> ReadedData {
        let file = Fl::open(path).unwrap();

        let end = min(file_len, end);
        let mut vec: Vec<u8> = vec![0; usize::try_from(end - start).unwrap()];

        file.read_at(&mut vec, start).unwrap();

        // todo: last_data and len
        return ReadedData {
            data: general_purpose::STANDARD.encode(vec),
            end,
            last_data: end >= file_len,
        };
    }
}

impl ProvideFile for FileService {
    fn get_tree(self: &FileService) -> Result<Directory, String> {
        let mut root_dir = Directory {
            name: "root".to_string(),
            path: Some(self.root_path.clone()),
            files: Some(Vec::new()),
            dirs: Some(Vec::new()),
        };
        self.get_tree_rec(&Path::new(&self.root_path), &mut root_dir);

        Ok(root_dir)
    }

    fn get_file_data(&self, start: u64, end: u64, file_key: String) -> Result<ReadedData, String> {
        // search file given the key
        let mut file: Option<String> = None;
        let mut file_len: Option<u64> = None;
        let mut hash_map = self.files_hash.lock().unwrap();
        if hash_map.contains_key(&file_key) {
            file = Some(hash_map.get(&file_key).unwrap().path.clone());
            file_len = Some(hash_map.get(&file_key).unwrap().size);
            hash_map.remove(&file_key);
        } else {
            let files = self.get_file_list(Path::new(&self.root_path));
            for f in files {
                if f.hash == file_key {
                    file = Some(f.path.clone().unwrap());
                    file_len = Some(f.size);
                    hash_map.insert(
                        f.hash,
                        PathCash {
                            time: Instant::now(),
                            path: f.path.clone().unwrap(),
                            size: f.size,
                        },
                    );
                }
            }
        }

        if file.is_none() {
            return Err("file don't exist".to_string());
        }

        // read data in, Read the interval [start, end)
        let data = self.read_data(file.unwrap(), file_len.unwrap(), start, end);

        // return the data
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn u8_to_string() {
        let b = b"asdf\xF0";
        let s = general_purpose::STANDARD.encode(b);
        println!("result: {s}");

        let bb = general_purpose::STANDARD.decode(s).unwrap();

        for i in [0..b.len()] {
            assert_eq!(bb[i.clone()], b[i]);
        }
    }
}
