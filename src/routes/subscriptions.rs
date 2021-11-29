use sqlx::Postgres;
use tide::{Request, Response, Result, StatusCode};

use sqlx::types::{chrono::Utc, Uuid};
use sqlx::Acquire;
use tide_sqlx::SQLxRequestExt;
use tracing_futures::Instrument;

#[derive(serde::Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(mut req: Request<()>) -> Result {
    tracing::info!("A new subscription request");
    if let Ok(result) = req.body_form().await {
        let new_sub: FormData = result;
        let request_id = Uuid::new_v4();
        let request_span = tracing::info_span!(
            "Adding a new subscriber",
            %request_id,
            subscriber_email = %new_sub.email,
            subscriber_name = %new_sub.name
        );
        let _request_span_guard = request_span.enter();

        let query_span = tracing::info_span!("Saving new subscriber details in the database");

        // Get the connection
        let mut pg_conn = req.sqlx_conn::<Postgres>().await;
        // Run the query
        match sqlx::query!(
            r#"
            insert into subscriptions (id, email, name, subscribed_at)  
            values ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            new_sub.email,
            new_sub.name,
            Utc::now()
        )
        .execute(pg_conn.acquire().await?)
        .instrument(query_span)
        .await
        {
            Ok(_) => {
                tracing::info!("Saved successfully");
                Ok(Response::new(StatusCode::Ok))
            }
            Err(e) => {
                tracing::error!("failed to execute query: {:?}", e);
                Ok(Response::new(StatusCode::BadRequest))
            }
        }
    } else {
        tracing::error!("the request was badly formatted");
        Ok(Response::new(StatusCode::BadRequest))
    }
}
