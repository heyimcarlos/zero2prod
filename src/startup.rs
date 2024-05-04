use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgConnection;

use crate::routes;

pub fn run(
    listener: std::net::TcpListener,
    connection: PgConnection,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
