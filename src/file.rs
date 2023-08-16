use std::fs::read_dir;
use std::path::Path;

use crate::ws::ws_message::{Directory, File, CopyRes};

pub trait ProvideFile {
    fn get_tree(&self) -> Result<Directory, String>;
    fn get_file_data(&self, start: i32, end: i32, file_key: String) -> Result<String, String>;
}

pub struct FileService {
    root_path: String,
}

impl FileService {
    pub fn new(root_path: String) -> FileService {
        FileService { root_path }
    }

    fn get_tree_rec(self: &FileService, path: &Path, dir: &mut Directory) {
        for entry in read_dir(path).unwrap() {
            let new_path = entry.unwrap().path();
            if new_path.is_dir() {
                let mut new_dir = Directory {
                    name: new_path.file_name().unwrap().to_str().unwrap().to_string(),
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
                };

                let mut files = dir.files.clone().unwrap();
                files.push(new_file);
                dir.files = Some(files);
            }
        }
    }
}

impl ProvideFile for FileService {
    fn get_tree(self: &FileService) -> Result<Directory, String> {
        let mut root_dir = Directory {
            name: "root".to_string(),
            files: Some(Vec::new()),
            dirs: Some(Vec::new()),
        };
        self.get_tree_rec(&Path::new(&self.root_path), &mut root_dir);

        Ok(root_dir)
    }
    fn get_file_data(&self, start: i32, end: i32, file_key: String) -> Result<String, String> {
        // search file given the key
        // read data in the interval [start, end)
        // return the data
        Ok("data test".to_string())
    }
}
