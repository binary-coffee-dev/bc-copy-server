use std::env::current_dir;
use std::fs::{create_dir, File, remove_dir_all};
use std::io::Write;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn create_dir_f(dir_path: String) {
    let path_res = format!("{}/{}", current_dir_path(), dir_path).to_string();
    create_dir(path_res).unwrap();
}

pub fn create_file_f(file_path: String) {
    let path_res = format!("{}/{}", current_dir_path(), file_path).to_string();
    let mut file = File::create(path_res).unwrap();
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(500000)
        .map(char::from)
        .collect();
    file.write_all(rand_string.as_bytes()).unwrap();
}

pub fn remove_dir_rec(path: String) {
    let path_res = format!("{}/{}", current_dir_path(), path).to_string();
    if remove_dir_all(path_res).is_ok() {}
}

pub fn current_dir_path() -> String {
    current_dir().unwrap().display().to_string()
}

pub fn gen_msg_id() -> i32 {
    thread_rng().gen_range(0..10000000)
}

pub fn setting_up_test_file_tree(data_test_path: String) {
    remove_dir_rec(data_test_path.clone());

    create_dir_f(data_test_path.clone());
    create_file_f(format!("{}/{}", data_test_path.clone(), "A.txt".to_string()).to_string());
    create_file_f(format!("{}/{}", data_test_path.clone(), "B.txt".to_string()).to_string());
    create_file_f(format!("{}/{}", data_test_path.clone(), "C.txt".to_string()).to_string());

    let dir1_path = format!("{}/{}", data_test_path, "dir1").to_string();
    create_dir_f(dir1_path.clone());
    create_file_f(format!("{}/{}", dir1_path.clone(), "A.txt".to_string()).to_string());
    create_file_f(format!("{}/{}", dir1_path.clone(), "B.txt".to_string()).to_string());
    create_file_f(format!("{}/{}", dir1_path.clone(), "C.txt".to_string()).to_string());
}
