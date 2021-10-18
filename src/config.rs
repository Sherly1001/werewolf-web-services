use rocket::Config;
use rocket::figment::{Figment, util::map};
use dotenv::dotenv;
use std::env;

pub fn from_env() -> Figment {
    dotenv().ok();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT environment variable should parse to an integer");

    let db = map! {
        "url" => env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found"),
    };

    Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", port))
        .merge(("databases", map!["postgres" => db]))
}
