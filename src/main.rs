use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::{
    configuration::get_configuration,
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
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to create connection pool.");
    let listener = std::net::TcpListener::bind((configuration.app.host, configuration.app.port))
        .unwrap_or_else(|_| panic!("Failed to bind port {}", &configuration.app.port));
    // Bubble up the io::Error  if we failed to bind the address
    // Otherwise call .await on the Server
    zero2prod::startup::run(listener, connection_pool)?.await?;
    Ok(())
}
