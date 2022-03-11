use crate::startup::State;
use async_std::sync::RwLockWriteGuard;
use sqlx::{Acquire, Postgres};
use tide::{Request, Response, StatusCode};
use tide_sqlx::{ConnectionWrapInner, SQLxRequestExt};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(
    mut pool: RwLockWriteGuard<'_, ConnectionWrapInner<Postgres>>,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool.acquire().await?)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "get subscriber_id from token", skip(pool))]
pub async fn get_subscriber_id_from_token(
    mut pool: RwLockWriteGuard<'_, ConnectionWrapInner<Postgres>>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id from subscription_tokens where subscription_token = $1"#,
        subscription_token,
    )
    .fetch_optional(pool.acquire().await?)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(req))]
pub async fn confirm(req: Request<State>) -> tide::Result {
    let params: Parameters = req.query()?;

    let id = match get_subscriber_id_from_token(
        req.sqlx_conn::<Postgres>().await,
        &params.subscription_token,
    )
    .await
    {
        Ok(id) => id,
        Err(_) => return Ok(Response::new(StatusCode::InternalServerError)),
    };

    match id {
        None => Ok(Response::new(StatusCode::NotFound)),
        Some(subscriber_id) => {
            if confirm_subscriber(req.sqlx_conn::<Postgres>().await, subscriber_id)
                .await
                .is_err()
            {
                return Ok(Response::new(StatusCode::InternalServerError));
            }
            Ok(Response::new(StatusCode::Ok))
        }
    }
}
