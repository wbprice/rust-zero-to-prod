use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[async_std::main]
async fn main() -> Result<(), tide::Error> {
    // panic if we can't fetch the configuration
    let configuration = get_configuration().expect("Failed to read configuration file.");

    // fetch port from configuration file
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Couldn't bind port");

    // create database connection pool
    let conn_string = configuration.database.connection_string();
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_string)
        .await?;

    // Start the server
    run(listener, pg_pool).await?;
    Ok(())
}
