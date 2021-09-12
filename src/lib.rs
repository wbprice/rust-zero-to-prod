use tide::{Request, Response};

pub fn run() -> tide::Server<()> {
    let mut app = tide::new();
    app.at("/health").get(health);
    app
}

async fn health(_req: Request<()>) -> tide::Result<Response> {
    Ok(Response::builder(200).build())
}
