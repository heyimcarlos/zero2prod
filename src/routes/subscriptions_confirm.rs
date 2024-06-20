use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::util::error_chain_fmt;
#[derive(serde::Deserialize)]
pub struct Parameters {
    pub subscription_token: String,
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
    .await
    .map_err(|err| {
        //  NOTE: We use std::fmt::Debug ({:?}) to get a raw view of the error, instead of
        // std::fmt::Display ({}) which displays a nicer error message (that could be displayed
        // to the end user)
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;

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
pub async fn confirm(params: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber_id = match get_subcriber_id_from_token(&pool, &params.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    match subscriber_id {
        Some(subscriber_id) => {
            if confirm_susbcriber(&pool, &subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
        // token doesn't exist
        None => return HttpResponse::Unauthorized().finish(),
    };
    HttpResponse::Ok().finish()
}
