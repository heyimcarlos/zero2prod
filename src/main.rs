use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
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
    // We use a connection pool, becase it allows for concurrent behavior. A pool contains multiple
    // connections, so when a query is to be executed, sqlx will borrow a connection from the pool,
    // if there's no connection available a new one will be created or wait until one is freed up.
    let connection_pool =
    // switch to `connect_lazy` because it will try to establish a connection only when the pool is
    // used for the first time.
        PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());
    let listener = std::net::TcpListener::bind((configuration.app.host, configuration.app.port))
        .unwrap_or_else(|_| panic!("Failed to bind port {}", &configuration.app.port));

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Failed to parse sender email");
    let email_client = EmailClient::new(configuration.email_client.base_url, sender_email);

    // Bubble up the io::Error  if we failed to bind the address
    // Otherwise call .await on the Server
    zero2prod::startup::run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
