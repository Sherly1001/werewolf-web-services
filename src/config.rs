use std::env;

#[derive(Clone)]
pub struct AppState {
    secret_key: String,
}

pub struct Config {
    pub port: u16,
    pub app_state: AppState,
}

pub fn load() -> std::io::Result<Config> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap();
    let secret_key = env::var("SECRET_KEY").unwrap();

    Ok(Config {
        port,
        app_state: AppState {
            secret_key,
        },
    })
}
