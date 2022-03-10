use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> Result<(), tide::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration file.");
    let app = Application::build(configuration).await?;
    app.run_until_stopped().await?;
    // Start the server
    Ok(())
}
