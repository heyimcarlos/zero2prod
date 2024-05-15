use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::{get_connection_pool, Application},
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

    let config = {
        let mut c = get_configuration().expect("Failed to parse configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        // Port 0 is special-cased at the OS level, when trying to bind port 0
        // a scan will be triggered to find an available port, and the bind to it.
        c.app.port = 0;
        c
    };

    // create and migrate db
    create_database(&config.database).await;

    let app = Application::build(config.clone())
        .await
        .expect("Failed to build app");

    let addr = format!("http://127.0.0.1:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        db_pool: get_connection_pool(&config.database),
        addr,
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
