use sqlx::Postgres;
use tide::{Request, Response, Result, StatusCode};

use sqlx::types::{chrono::Utc, Uuid};
use sqlx::Acquire;
use tide_sqlx::SQLxRequestExt;

#[derive(serde::Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(mut req: Request<()>) -> Result {
    if let Ok(result) = req.body_form().await {
        let mut pg_conn = req.sqlx_conn::<Postgres>().await;
        let new_sub: FormData = result;

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
        .await
        {
            Ok(_) => Ok(Response::new(StatusCode::Ok)),
            Err(e) => {
                println!("Failed to execute query: {}", e);
                Ok(Response::new(StatusCode::BadRequest))
            }
        }
    } else {
        Ok(Response::new(StatusCode::BadRequest))
    }
}
