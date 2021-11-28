#[macro_use]
extern crate diesel;

use actix::{Actor, Addr};
use actix_web::{
    error::ErrorBadRequest,
    middleware, web, App, HttpRequest, HttpServer,
};

mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod schema;
mod ws;

use auth::Auth;
use error::Res;
use ws::{ChatServer, WsClient};

async fn notfound_handle() -> Res {
    Err(ErrorBadRequest("resource not found"))
}

#[actix_web::get("/ws/")]
async fn ws_handler(
    auth: Auth,
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> Res {
    actix_web_actors::ws::start(
        WsClient::new(auth.user_id, srv.as_ref().clone()),
        &req,
        stream,
    )
}

#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = config::load()?;
    let app_state = config.app_state.clone();
    let db_pool = config.db_pool.clone();

    let chat_server = ChatServer::new(app_state.clone(), db_pool.clone())
        .start();

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();

        App::new()
            .wrap(error::ResErrWrap)
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(chat_server.clone()))
            .data(db_pool.clone())
            .service(
                web::scope("/users")
                    .service(routes::user::create)
                    .service(routes::user::login)
                    .service(routes::user::get_all)
                    .service(routes::user::get_info)
            )
            .service(ws_handler)
            .default_service(web::to(notfound_handle))
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await
}
