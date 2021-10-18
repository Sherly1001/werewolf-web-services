#[macro_use]
extern crate rocket;
extern crate dotenv;

mod config;

#[rocket::main]
pub async fn run() {
    rocket::custom(config::from_env())
        .mount("/", routes![])
        .launch()
        .await.unwrap();
}
