use crate::startup::State;
use tide::{Request, Response, Result, StatusCode};

pub async fn health_check(_req: Request<State>) -> Result {
    Ok(Response::new(StatusCode::Ok))
}
