use crate::routes::{health_check, subscribe};
use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use tide::log::LogMiddleware;
use tide::prelude::*;
use tide_sqlx::SQLxMiddleware;

pub async fn run(listener: TcpListener, pool: PgPool) -> tide::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut app = tide::new();
    app.with(SQLxMiddleware::from(pool));
    app.with(LogMiddleware::new());
    app.at("/health").get(health_check);
    app.at("/subscriptions").post(subscribe);
    let mut listener = app.bind(listener).await?;
    listener.accept().await?;
    Ok(())
}
