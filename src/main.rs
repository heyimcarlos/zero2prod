use zero2prod::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

// Procedural macro which initializes an async runtime that block on (drives) HttpServer::run
// returned futures to completion.
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    //  NOTE: Configure tracing
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    tracing::info!(
        "SETTINGS configuration.email_client! {:?}",
        configuration.email_client
    );

    let server = Application::build(configuration).await?;
    server.run_until_stopped().await?;
    Ok(())
}
