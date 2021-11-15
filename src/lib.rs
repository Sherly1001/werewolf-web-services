use actix_web::{
    App,
    HttpServer,
    HttpResponse,
    web,
};


#[actix_web::main]
pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::to(|| HttpResponse::Ok()))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
