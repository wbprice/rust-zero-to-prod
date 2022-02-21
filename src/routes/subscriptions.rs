use async_std::sync::RwLockWriteGuard;
use sqlx::types::{chrono::Utc, Uuid};
use sqlx::Acquire;
use sqlx::Postgres;
use tide::{Request, Response, StatusCode};
use tide_sqlx::{ConnectionWrapInner, SQLxRequestExt};
use tracing::{field, Span};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(name = "Saving subscriber details in the database", skip(pool, form))]
pub async fn insert_subscriber(
    mut pool: RwLockWriteGuard<'_, ConnectionWrapInner<Postgres>>,
    form: FormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            insert into subscriptions (id, email, name, subscribed_at)  
            values ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
        let new_sub: FormData = result;

        let span = Span::current();
        span.record("subscriber_email", &new_sub.email.as_str());
        span.record("subscriber_name", &new_sub.name.as_str());

        match insert_subscriber(pg_conn, new_sub).await {
            Ok(_) => Ok(Response::new(StatusCode::Ok)),
            Err(_) => Ok(Response::new(StatusCode::BadRequest)),
        }
    } else {
        tracing::error!("Couldn't parse input");
        Ok(Response::new(StatusCode::BadRequest))
    }
}
