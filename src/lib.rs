use std::net::TcpListener;
use tide::prelude::*;
use tide::{Request, Response, StatusCode};

async fn health(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

pub async fn run(listener: TcpListener) -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/health").get(health);
    let mut listener = app.bind(listener).await?;
    listener.accept().await?;
    Ok(())
}
