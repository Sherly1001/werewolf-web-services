use actix_web::{App, HttpServer, http::StatusCode, web};


mod error;
use error::ResErr;

async fn notfound_handle() -> error::Res {
    Err(ResErr::new(StatusCode::BAD_REQUEST, "resource not found".to_string()))
}

#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .default_service(web::to(notfound_handle))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
