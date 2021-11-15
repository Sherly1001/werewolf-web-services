use actix_web::{http::StatusCode, middleware, web, App, HttpServer};

mod config;
mod error;
use error::{Res, ResErr};

async fn notfound_handle() -> Res {
    Err(ResErr::new(
        StatusCode::BAD_REQUEST,
        "resource not found".to_string(),
    ))
}

#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let config = config::load()?;
    let app_state = config.app_state.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(app_state.clone())
            .default_service(web::to(notfound_handle))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}
