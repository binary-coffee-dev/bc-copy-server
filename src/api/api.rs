use serde::Deserialize;

#[derive(Deserialize)]
struct Client {
    name: String,
}

#[get("/api/clients")]
pub fn client_api() -> &'static str {
    "Hello, world!"
}

// #[get("/api/clients")]
// pub fn client_api() -> &'static str {
//     "Hello, world!"
// }
