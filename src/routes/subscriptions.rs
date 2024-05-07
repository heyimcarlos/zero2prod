use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    web::Form(form): web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let request_id = Uuid::new_v4();
    // NOTE: Every interaction with external systems should be **closely** monitored.
    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber",
        request_id,
        form.email,
        form.name
    );
    log::info!(
        "request_id {} - Saving new subscriber details to the database",
        request_id
    );
    let query = sqlx::query!(
        //  TODO: Raw string literals ignore special characters and escapes. r#""# (raw string literal) documented on: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals.
        "INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await;

    match query {
        Ok(_) => {
            log::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            // We use std::fmt::Debug ({:?}) to get a raw view of the error, instead of
            // std::fmt::Display ({}) which displays a nicer error message (that could be displayed
            // to the end user)
            log::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                err
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
