use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer};

pub fn run(listener: std::net::TcpListener) -> Result<Server, std::io::Error> {
    dbg!(&listener);
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subscriptions(_request: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}
