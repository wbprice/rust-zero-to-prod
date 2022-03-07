use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> Result<(), tide::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // panic if we can't fetch the configuration
    let configuration = get_configuration().expect("Failed to read configuration file.");

    // fetch port from configuration file
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address).expect("Couldn't bind port");

    // create database connection pool
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to create Postgres connection pool.");

    // Start the server
    run(listener, pg_pool).await?;
    Ok(())
}
