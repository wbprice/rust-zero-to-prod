use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use crate::startup::State;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use sqlx::types::{chrono::Utc, Uuid};
use sqlx::Acquire;
use sqlx::Postgres;
use std::convert::{TryFrom, TryInto};
use tide::{Request, Response, StatusCode};
use tide_sqlx::SQLxRequestExt;
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

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Saving subscriber details in the database",
    skip(transaction, new_sub)
)]
pub async fn insert_subscriber(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    new_sub: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
            insert into subscriptions (id, email, name, subscribed_at, status)
            values ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        subscriber_id,
        new_sub.email.as_ref(),
        new_sub.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store subscription tokens in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &String,
    subscription_token: &String,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );

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
        let email_client = &req.state().email_client;
        let base_url = &req.state().base_url;

        let form: FormData = result;
        let span = Span::current();
        span.record("subscriber_email", &form.email.as_str());
        span.record("subscriber_name", &form.name.as_str());

        let subscription_token = generate_subscription_token();
        let new_subscriber = match form.try_into() {
            Ok(subscriber) => subscriber,
            Err(_) => return Ok(Response::new(StatusCode::BadRequest)),
        };

        let mut pool = req.sqlx_conn::<Postgres>().await;
        let mut transaction = match pool.begin().await {
            Ok(transaction) => transaction,
            Err(_) => return Ok(Response::new(StatusCode::InternalServerError)),
        };

        let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
            Ok(subscriber_id) => subscriber_id,
            Err(_) => return Ok(Response::new(StatusCode::BadRequest)),
        };

        if store_token(&mut transaction, subscriber_id, &subscription_token)
            .await
            .is_err()
        {
            return Ok(Response::new(StatusCode::BadRequest));
        }

        if transaction.commit().await.is_err() {
            return Ok(Response::new(StatusCode::InternalServerError));
        }

        if send_confirmation_email(&email_client, new_subscriber, base_url, &subscription_token)
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
