use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

// Create a static item `TRACING` which is available for the entire duration of the program
// it has a static lifetime. `once_cell::sync::Lazy` is a value which is initialized on the
// first access. Ensures that the `tracing` stack is only initialized once.
static TRACING: once_cell::sync::Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test".to_string();
    let default_filter_level = "debug".to_string();

    // Initialize tracing
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        // if `TEST_LOG` is not set, send logs to the void.
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // the first time `spawn_app` is invoked, `TRACING` will be executed.
    // All other invocations will skip execution.
    Lazy::force(&TRACING);

    // Port 0 is special-cased at the OS level, when trying to bind port 0
    // a scan will be triggered to find an available port, and the bind to it.
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = get_configuration().expect("Failed to get configuration.");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = create_database(&config.database).await;

    let sender_email = config
        .email_client
        .sender()
        .expect("Failed to parse sender email");

    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.auth_token,
        timeout,
    );

    let server = zero2prod::startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp {
        addr: address,
        db_pool: connection_pool,
    }
}

async fn create_database<'a>(config: &'a DatabaseSettings) -> PgPool {
    // Connect to db server.
    let mut conn = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to the database.");

    // Create test db.
    conn.execute(format!(r#"CREATE DATABASE "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // println!("db string: {}", conn_string);

    // create db pool & migrate.
    let conn_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to create database pool.");
    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("Failed to migrate the database.");
    conn_pool
}
