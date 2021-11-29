use crate::routes::{health_check, subscribe};
use sqlx::PgPool;
use std::net::TcpListener;
use tide::prelude::*;
use tide_sqlx::SQLxMiddleware;

pub async fn run(listener: TcpListener, pool: PgPool) -> tide::Result<()> {
    let mut app = tide::new();
    app.with(SQLxMiddleware::from(pool));
    app.at("/health").get(health_check);
    app.at("/subscriptions").post(subscribe);
    let mut listener = app.bind(listener).await?;
    listener.accept().await?;
    Ok(())
}
