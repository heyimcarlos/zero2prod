use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::AppBaseUrl,
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

// generate a random 25-characters-long case-sensitive subscription token
fn gen_subscription_token() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    // `error()` defines what `Display` is printing.
    // interpolation `"{0}"` works similarly to `self.0`
    //  NOTE: We don't use `source` or `from` here because `String` does not impl the `Error`
    //  trait
    #[error("{0}")]
    ValidationError(String),
    // `error(transparent)` delegates `Display` and `source` impl to the type wrapped by
    // `UnexpectedError`
    #[error("{1}")]
    // `from` automatically derives an impl for the `From` trait (e.g. From<StoreTokenError> for
    // SubscribeError) and applies `#[source]`
    // `source` is used to denote what should be returned as the root case in Error::source
    UnexpectedError(#[source] Box<dyn std::error::Error>, String),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<AppBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    // manually map the error
    let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = pool.begin().await.map_err(|err| {
        SubscribeError::UnexpectedError(
            Box::new(err),
            "Failed to acquire Postgres connection from the pool".into(),
        )
    })?;
    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction)
        .await
        .map_err(|err| {
            SubscribeError::UnexpectedError(
                Box::new(err),
                "Failed to insert new subscriber in the database".into(),
            )
        })?;
    let subscription_token = gen_subscription_token();
    store_token(subscriber_id, &subscription_token, &mut transaction)
        .await
        .map_err(|err| {
            SubscribeError::UnexpectedError(
                Box::new(err),
                "Failed to store the confirmation token for a new \
            subscriber"
                    .into(),
            )
        })?;
    transaction.commit().await.map_err(|err| {
        SubscribeError::UnexpectedError(
            Box::new(err),
            "Failed to commit SQL transaction to store a new subscriber".into(),
        )
    })?;
    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .map_err(|err| {
        SubscribeError::UnexpectedError(Box::new(err), "Failed to send confirmation email".into())
    })?;
    Ok(HttpResponse::Ok().finish())
}

// attach instrumentation
#[tracing::instrument(name = "Saving new subscriber details to the database", skip_all)]
async fn insert_subscriber<'a>(
    new_subscriber: &'a NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        //  TODO: Raw string literals ignore special characters and escapes. r#""# (raw string literal) documented on: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals.
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')",
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );
    transaction.execute(query).await.map_err(|err| {
        //  NOTE: We use std::fmt::Debug ({:?}) to get a raw view of the error, instead of
        // std::fmt::Display ({}) which displays a nicer error message (that could be displayed
        // to the end user)
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;
    Ok(subscriber_id)
}

// New error type for `store_token`
pub struct StoreTokenError(pub sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token"
        )
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self.0, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // the compiler transparently casts `&sqlx::Error` into a `&dyn Error`
        Some(&self.0)
    }
}

// `store_token` is a fallible operation
#[tracing::instrument(name = "Saving new subscription_token to the database", skip_all)]
async fn store_token(
    subscriber_id: Uuid,
    subscription_token: &str,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        "INSERT INTO subscription_tokens (subscriber_id, subscription_token)
        VALUES ($1, $2)",
        subscriber_id,
        subscription_token
    );
    transaction.execute(query).await.map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        StoreTokenError(err)
    })?;
    Ok(())
}

#[tracing::instrument(name = "Sending confirmation email to subscriber", skip_all)]
async fn send_confirmation_email<'a>(
    email_client: &'a EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &'a str,
    subscription_token: &'a str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
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

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
