use env_logger::{Builder, Env};
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;

// Procedural macro which initializes an async runtime that block on (drives) HttpServer::run
// returned futures to completion.
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // init() calls `set_logger`. Fallback to printing all logs at info-level or above
    // if RUST_LOG env variable has not been set.
    Builder::from_env(Env::default().default_filter_or("info")).init();
    let configuration = get_configuration().expect("Failed to read configuration.");
    // We use a connection pool, becase it allows for concurrent behavior. A pool contains multiple
    // connections, so when a query is to be executed, sqlx will borrow a connection from the pool,
    // if there's no connection available a new one will be created or wait until one is freed up.
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to create connection pool.");
    let listener = std::net::TcpListener::bind(("127.0.0.1", configuration.application_port))
        .unwrap_or_else(|_| panic!("Failed to bind port {}", &configuration.application_port));
    // Bubble up the io::Error  if we failed to bind the address
    // Otherwise call .await on the Server
    zero2prod::startup::run(listener, connection_pool)?.await
}
