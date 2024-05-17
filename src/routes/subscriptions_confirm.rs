use actix_web::{web, HttpResponse};

use crate::startup::AppBaseUrl;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn confirm(
    base_url: web::Data<AppBaseUrl>,
    params: web::Query<Parameters>,
) -> HttpResponse {
    // params.subscription_token;
    HttpResponse::Ok().finish()
}
