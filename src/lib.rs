
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn subscribe(web::Form(_form): web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
pub mod routes;
pub mod startup;
