use std::sync::{Mutex, Once};

use actix_web::{web::Data, test, App};
use lazy_static::lazy_static;

use test_utils::{current_dir_path, setting_up_test_file_tree};
use cs::api::api::{Client, client_api_endpoint, create_client_endpoint, delete_client_endpoint};
use cs::data::DataService;
use cs::file::FileService;

static BEFORE_ALL: Once = Once::new();

lazy_static! {
    static ref DATA_INS: Data<Mutex<DataService>> = Data::new(Mutex::new(DataService::new()));

    // test with FileService
    static ref FILE_INS: Data<Mutex<FileService>> = Data::new(Mutex::new(FileService::new(
        format!("{}/{}", current_dir_path(), "data-test")
    )));
}

fn before_all() {
    BEFORE_ALL.call_once(|| {
        // testing files
        setting_up_test_file_tree("data-test".to_string());
    });
}

fn create_mock_clients(clients: Vec<String>) {
    for name in clients {
        let data_service = DATA_INS.clone();
        data_service.lock().unwrap().new_client(Client {
            id: None,
            key: None,
            name: Some(name),
        });
    }
}

#[test]
async fn list_of_clients_request_test() {
    before_all();

    let client_name = "list_of_clients_request_test".to_string();
    create_mock_clients(vec![client_name.clone()]);

    let app = test::init_service(
        App::new()
            .app_data(Data::clone(&DATA_INS))
            .service(client_api_endpoint)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/clients")
        .to_request();
    let clients_resp: Vec<Client> = test::call_and_read_body_json(&app, req).await;

    let mut found = false;
    for cli in clients_resp.clone() {
        if cli.name.clone().unwrap() == client_name {
            found = true;
            break;
        }
    }

    assert!(clients_resp.len() > 0);
    assert!(found);
}

#[test]
async fn create_new_client_test() {
    before_all();

    let app = test::init_service(
        App::new()
            .app_data(Data::clone(&DATA_INS))
            .service(create_client_endpoint)
    ).await;

    let new_cli_name = "new_cli_name_api".to_string();
    let new_cli = Client {
        id: None,
        key: None,
        name: Some(new_cli_name.clone()),
    };

    let req = test::TestRequest::post()
        .uri("/api/clients")
        .set_form(new_cli)
        .to_request();
    let new_clients_resp: Client = test::call_and_read_body_json(&app, req).await;

    assert_eq!(new_clients_resp.name.clone().unwrap(), new_cli_name.clone());
    let cli = DATA_INS.lock().unwrap().get_client_by_name(new_cli_name);
    assert!(cli.is_some());
}

#[test]
async fn delete_client_request_test() {
    before_all();

    let app = test::init_service(
        App::new()
            .app_data(Data::clone(&DATA_INS))
            .service(delete_client_endpoint)
    ).await;

    let to_remove_cli_name = "new_cli_name_api".to_string();
    create_mock_clients(vec![to_remove_cli_name.clone()]);

    let cli_id;
    {
        cli_id = DATA_INS.lock().unwrap().
            get_client_by_name(to_remove_cli_name.clone()).unwrap().id;
    }

    assert!(cli_id.is_some());

    let req = test::TestRequest::delete()
        .uri(format!("/api/clients/{}", cli_id.unwrap()).as_str())
        .to_request();
    let removed_clients_resp: Client = test::call_and_read_body_json(&app, req).await;

    assert_eq!(removed_clients_resp.name.unwrap(), to_remove_cli_name.clone());
    let cli = DATA_INS.lock().unwrap().get_client_by_name(to_remove_cli_name);
    assert!(cli.is_none());
}
