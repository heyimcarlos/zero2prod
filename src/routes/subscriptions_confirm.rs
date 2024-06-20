use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::util::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmationError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("There is no subscriber associated with the provided token")]
    UnknownToken,
}

impl std::fmt::Debug for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmationError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ConfirmationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConfirmationError::UnknownToken => StatusCode::UNAUTHORIZED,
        }
    }
}

async fn get_subcriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id AS id FROM subscription_tokens \
        WHERE subscription_token = $1",
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.id))
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip_all)]
async fn confirm_susbcriber(pool: &PgPool, subscriber_id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;
    Ok(())
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn confirm(
    params: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmationError> {
    let subscriber_id = get_subcriber_id_from_token(&pool, &params.subscription_token)
        .await
        .context("Failed to get subscriber from token")?;
    confirm_susbcriber(&pool, &subscriber_id.unwrap())
        .await
        .context("Failed to confirm subscriber")?;

    Ok(HttpResponse::Ok().finish())
}
