use crate::startup::State;
use tide::{Request, Response, Result, StatusCode};

#[tracing::instrument(name = "Confirm a pending subscriber", skip(req))]
pub async fn confirm(mut req: Request<State>) -> Result {
    Ok(Response::new(StatusCode::Ok))
}
