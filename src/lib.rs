#[macro_use]
extern crate rocket;
extern crate dotenv;
extern crate jsonwebtoken;
extern crate crypto;

#[macro_use]
extern crate diesel;


mod config;
mod db;
mod schema;
mod models;
mod routes;
mod auth;
mod errors;

use rocket::serde::json::{Value, serde_json::json};

#[catch(403)]
fn forbidden() -> Value {
    json!({
        "status": "error",
        "message": "you don't have permission to access",
    })
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "message": "resource was not found",
    })
}

#[rocket::main]
pub async fn run() {
    rocket::custom(config::from_env())
        .mount("/users", routes![
            routes::users::create,
            routes::users::login,
            routes::users::get_users,
        ])
        .register("/", catchers![forbidden, not_found])
        .manage(config::AppState::new())
        .attach(db::Conn::fairing())
        .launch()
        .await.unwrap();
}
