use crate::startup::State;
use fake::faker::lorem::raw::Paragraphs;
use serde::Deserialize;
use tide::{Request, Response, Result, StatusCode};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
} 

#[tracing::instrument(name = "Confirm a pending subscriber", skip(req))]
pub async fn confirm(mut req: Request<State>) -> Result {
    let params: Parameters = req.query()?;

    Ok(Response::new(StatusCode::Ok))
}

