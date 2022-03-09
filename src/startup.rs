use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use async_std::sync::Arc;
use sqlx::PgPool;
use std::net::TcpListener;
use tide::prelude::*;
use tide_sqlx::SQLxMiddleware;
use tide_tracing::TraceMiddleware;

#[derive(Clone)]
pub struct State {
    pub email_client: Arc<EmailClient>,
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
