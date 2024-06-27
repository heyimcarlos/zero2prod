use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes,
};

// new type to hold the application and its port
pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    // convert `build` into an `App` struct constructor
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&config.database);

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

        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = std::net::TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        // Bubble up the io::Error  if we failed to bind the address
        // Otherwise call .await on the Server
        let server = run(
            listener,
            connection_pool,
            email_client,
            config.application.base_url,
        )?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // a more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    // We use a connection pool, becase it allows for concurrent behavior. A pool contains multiple
    // connections, so when a query is to be executed, sqlx will borrow a connection from the pool,
    // if there's no connection available a new one will be created or wait until one is freed up.

    // switch to `connect_lazy` because it will try to establish a connection only when the pool is
    // used for the first time.
    PgPoolOptions::new().connect_lazy_with(config.with_db())
}

pub struct AppBaseUrl(pub String);

fn run(
    listener: std::net::TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let base_url = web::Data::new(AppBaseUrl(base_url));
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            // instead of Logger:default()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            .route("/subscriptions/confirm", web::get().to(routes::confirm))
            .route("/newsletters", web::post().to(routes::broadcast))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
