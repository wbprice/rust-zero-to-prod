use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::State;
use async_std::sync::RwLockWriteGuard;
use serde::Deserialize;
use sqlx::types::{chrono::Utc, Uuid};
use sqlx::Acquire;
use sqlx::Postgres;
use std::convert::{TryFrom, TryInto};
use tide::{Request, Response, StatusCode};
use tide_sqlx::{ConnectionWrapInner, SQLxRequestExt};
use tracing::{field, Span};

#[derive(Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
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
            insert into subscriptions (id, email, name, subscribed_at, status)
            values ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        Uuid::new_v4(),
        new_sub.email.as_ref(),
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
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &String,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{}/subscriptions/confirm", base_url);

    let plain_body = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = &format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(new_subscriber.email, "Welcome", html_body, plain_body)
        .await
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(req),
    fields(
        subscriber_email = field::Empty,
        subscriber_name = field::Empty
    )
)]
pub async fn subscribe(mut req: Request<State>) -> tide::Result {
    if let Ok(result) = req.body_form().await {
        let form: FormData = result;
        let email_client = &req.state().email_client;
        let base_url = &req.state().base_url;
        let pg_conn = req.sqlx_conn::<Postgres>().await;
        let span = Span::current();
        span.record("subscriber_email", &form.email.as_str());
        span.record("subscriber_name", &form.name.as_str());

        let new_subscriber = match form.try_into() {
            Ok(subscriber) => subscriber,
            Err(_) => return Ok(Response::new(StatusCode::BadRequest)),
        };

        if insert_subscriber(pg_conn, &new_subscriber).await.is_err() {
            return Ok(Response::new(StatusCode::BadRequest));
        }

        if send_confirmation_email(&email_client, new_subscriber, base_url)
            .await
            .is_err()
        {
            return Ok(Response::new(StatusCode::InternalServerError));
        }
        Ok(Response::new(StatusCode::Ok))
    } else {
        tracing::error!("Couldn't parse input");
        Ok(Response::new(StatusCode::BadRequest))
    }
}
