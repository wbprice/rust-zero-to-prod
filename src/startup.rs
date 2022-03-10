use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, subscribe};
use async_std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tide::prelude::*;
use tide_sqlx::SQLxMiddleware;
use tide_tracing::TraceMiddleware;

#[derive(Clone)]
pub struct State {
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn register_server(
    pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<tide::Server<State>, std::io::Error> {
    let mut app = tide::with_state(State {
        email_client: Arc::new(email_client),
        base_url: base_url,
    });
    app.with(SQLxMiddleware::from(pool));
    app.with(TraceMiddleware::new());
    app.at("/health").get(health_check);
    app.at("/subscriptions").post(subscribe);
    app.at("/subscriptions/confirm").get(confirm);
    Ok(app)
}

pub struct Application {
    port: u16,
    server: tide::Server<State>,
    listener: TcpListener,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender address");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        // fetch port from configuration file
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).expect("Couldn't bind port");
        let port = listener.local_addr().unwrap().port();
        let server = register_server(
            connection_pool,
            email_client,
            configuration.application.base_url,
        )
        .unwrap();

        Ok(Self {
            port,
            server,
            listener,
        })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        let mut listener = self.server.bind(self.listener).await?;
        listener.accept().await
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
