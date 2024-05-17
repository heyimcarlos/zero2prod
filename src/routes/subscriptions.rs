use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

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
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    if insert_subscriber(&new_subscriber, &pool).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };
    HttpResponse::Ok().finish()
}

// attach instrumentation
#[tracing::instrument(name = "Saving new subscriber details to the database", skip_all)]
async fn insert_subscriber<'a>(
    new_subscriber: &'a NewSubscriber,
    pool: &'a PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        //  TODO: Raw string literals ignore special characters and escapes. r#""# (raw string literal) documented on: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals.
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')",
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

#[tracing::instrument(name = "Sending confirmation email to subscriber", skip_all)]
async fn send_confirmation_email<'a>(
    email_client: &'a EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://there-is-no-domain.com/subscriptions/confirm";
    let subject = "subject";
    let html_body = format!(
        "Welcome to our newsletter!<br>\
            Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, subject, &html_body, &plain_body)
        .await
}
