use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use diesel::PgConnection;
use snowflake::SnowflakeIdGenerator;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn get_conn(
    pool: actix_web::web::Data<DbPool>,
) -> PooledConnection<ConnectionManager<PgConnection>> {
    loop {
        match pool.get_timeout(std::time::Duration::from_secs(3)) {
            Ok(conn) => break conn,
            _ => continue,
        }
    }
}

use std::env;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub secret_key: String,
    pub bot_prefix: String,
    pub bot_id: i64,
    pub id_generatator: Arc<Mutex<SnowflakeIdGenerator>>,
}

pub struct Config {
    pub port: u16,
    pub app_state: AppState,
    pub db_pool: DbPool,
}

pub fn load() -> std::io::Result<Config> {
    let port = env::var("PORT")
        .unwrap_or(8080.to_string())
        .parse()
        .unwrap();
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY");
    let bot_prefix = env::var("BOT_PREFIX").unwrap_or("!".to_string());
    let bot_id = env::var("BOT_ID").expect("BOT_ID").parse().unwrap();

    let connspec = env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool_size = env::var("DATABASE_POOL_SIZE")
        .unwrap_or(10.to_string())
        .parse()
        .unwrap();
    let timeout = env::var("DATABASE_TIMEOUT")
        .unwrap_or(10.to_string())
        .parse()
        .unwrap();

    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let db_pool = r2d2::Pool::builder()
        .max_size(pool_size)
        .connection_timeout(std::time::Duration::from_secs(timeout))
        .build(manager)
        .expect("Failed to create pool.");

    let id_generatator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(1, 1)));

    Ok(Config {
        port,
        app_state: AppState {
            secret_key,
            bot_prefix,
            bot_id,
            id_generatator,
        },
        db_pool,
    })
}
