use rocket::Config;
use rocket::figment::{Figment, util::map};
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
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT environment variable should parse to an integer");

    let db = rocket_sync_db_pools::Config {
        url: env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found"),
        pool_size: env::var("DATABASE_POOL_SIZE")
            .map(|size| size.parse().expect("pool_size not u32 type"))
            .unwrap_or_else(|_| 5),
        timeout: env::var("DATABASE_TIMEOUT")
            .map(|timout| timout.parse().expect("timeout not u8 type"))
            .unwrap_or_else(|_| 5),
    };

    Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", port))
        .merge(("databases", map!["postgres" => db]))
}
