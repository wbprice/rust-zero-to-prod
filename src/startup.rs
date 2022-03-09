use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use async_std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tide::prelude::*;
use tide_sqlx::SQLxMiddleware;
use tide_tracing::TraceMiddleware;

#[derive(Clone)]
pub struct State {
    pub email_client: Arc<EmailClient>,
}

pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {
    // create database connection pool
    let pg_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // Configure email client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    // fetch port from configuration file
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Couldn't bind port");

    run(listener, pg_pool, email_client)
}

pub async fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
) -> tide::Result<()> {
    let mut app = tide::with_state(State {
        email_client: Arc::new(email_client),
    });
    app.with(SQLxMiddleware::from(pool));
    app.with(TraceMiddleware::new());
    app.at("/health").get(health_check);
    app.at("/subscriptions").post(subscribe);
    let mut listener = app.bind(listener).await?;
    listener.accept().await?;
    Ok(())
}
