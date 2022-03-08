use crate::domain::{NewSubscriber, SubscriberName};
use async_std::sync::RwLockWriteGuard;
use serde::Deserialize;
use sqlx::types::{chrono::Utc, Uuid};
use sqlx::Acquire;
use sqlx::Postgres;
use tide::{Request, Response, StatusCode};
use tide_sqlx::{ConnectionWrapInner, SQLxRequestExt};
use tracing::{field, Span};

#[derive(Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Saving subscriber details in the database",
    skip(pool, new_sub)
)]
pub async fn insert_subscriber(
    mut pool: RwLockWriteGuard<'_, ConnectionWrapInner<Postgres>>,
    new_sub: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            insert into subscriptions (id, email, name, subscribed_at)  
            values ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        new_sub.email,
        new_sub.name.as_ref(),
        Utc::now()
    )
    .execute(pool.acquire().await?)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(req),
    fields(
        subscriber_email = field::Empty,
        subscriber_name = field::Empty
    )
)]
pub async fn subscribe(mut req: Request<()>) -> tide::Result {
    if let Ok(result) = req.body_form().await {
        let pg_conn = req.sqlx_conn::<Postgres>().await;
        let form: FormData = result;
        let span = Span::current();
        span.record("subscriber_email", &form.email.as_str());
        span.record("subscriber_name", &form.name.as_str());

        let new_subscriber = NewSubscriber {
            email: form.email,
            name: SubscriberName::parse(form.name).expect("Name validation failed."),
        };

        match insert_subscriber(pg_conn, &new_subscriber).await {
            Ok(_) => Ok(Response::new(StatusCode::Ok)),
            Err(_) => Ok(Response::new(StatusCode::BadRequest)),
        }
    } else {
        tracing::error!("Couldn't parse input");
        Ok(Response::new(StatusCode::BadRequest))
    }
}
