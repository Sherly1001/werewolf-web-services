use rocket::Config;
use rocket::figment::{Figment, util::map};
use dotenv::dotenv;
use std::env;

pub const TOKEN_PREFIX: &'static str = "Token ";

pub struct AppState {
    pub secret: Vec<u8>,
}

impl AppState {
    pub fn new() -> Self {
        let secret = env::var("SECRET_KEY").unwrap_or_else(|err| {
            if cfg!(debug_assertions) {
                println!("Debug mode, use SherNoob secret key");
                "SherNoob".to_string()
            } else {
                panic!("No SECRET_KEY environment variable found: {:?}", err)
            }
        });

        Self {
            secret: secret.into_bytes()
        }
    }
}

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
