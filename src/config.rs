use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use snowflake::SnowflakeIdGenerator;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use std::env;

#[derive(Clone)]
pub struct AppState {
    pub secret_key: String,
    pub id_generatator: SnowflakeIdGenerator,
}

pub struct Config {
    pub port: u16,
    pub app_state: AppState,
    pub db_pool: DbPool,
}

pub fn load() -> std::io::Result<Config> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap();
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY");

    let connspec = env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let id_generatator = SnowflakeIdGenerator::new(1, 1);

    Ok(Config {
        port,
        app_state: AppState {
            secret_key,
            id_generatator,
        },
        db_pool,
    })
}
