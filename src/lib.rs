use tide::{Request, Response, StatusCode};

async fn health(_req: Request<()>) -> tide::Result {
    Ok(Response::new(StatusCode::Ok))
}

pub async fn run(address: &str) -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/health").get(health);
    app.listen(address).await?;
    Ok(())
}
