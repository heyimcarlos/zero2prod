use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use zero2prod::run;

// Procedural macro which initializes an async runtime that block on (drives) HttpServer::run
// returned futures to completion.
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    run().await
}
