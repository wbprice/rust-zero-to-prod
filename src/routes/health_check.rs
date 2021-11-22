use tide::{Request, Response, Result, StatusCode};

pub async fn health_check(_req: Request<()>) -> Result {
    Ok(Response::new(StatusCode::Ok))
}
