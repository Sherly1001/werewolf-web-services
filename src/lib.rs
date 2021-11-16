#[macro_use]
extern crate diesel;

use actix_web::{
    error::ErrorBadRequest,
    middleware, web, App, HttpServer,
};

mod config;
mod db;
mod error;
mod models;
mod routes;
mod schema;
use error::Res;

async fn notfound_handle() -> Res {
    Err(ErrorBadRequest("resource not found"))
}

#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = config::load()?;
    let app_state = config.app_state.clone();
    let db_pool = config.db_pool.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(error::ResErrWrap)
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::default())
            .app_data(web::Data::new(app_state.clone()))
            .data(db_pool.clone())
            .service(
                web::scope("/users")
                    .service(routes::user::create),
            )
            .default_service(web::to(notfound_handle))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}
