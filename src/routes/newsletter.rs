use actix_web::HttpResponse;

pub async fn broadcast() -> HttpResponse {
    HttpResponse::Ok().finish()
}
