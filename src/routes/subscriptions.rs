use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(Self { name, email })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&new_subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// attach instrumentation
#[tracing::instrument(name = "Saving new subscriber details to the database", skip_all)]
async fn insert_subscriber<'a>(
    new_subscriber: &'a NewSubscriber,
    pool: &'a PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        //  TODO: Raw string literals ignore special characters and escapes. r#""# (raw string literal) documented on: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals.
        "INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|err| {
        //  NOTE: We use std::fmt::Debug ({:?}) to get a raw view of the error, instead of
        // std::fmt::Display ({}) which displays a nicer error message (that could be displayed
        // to the end user)
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;
    Ok(())
}
