use sqlx::PgPool;
use zero2prod::configuration::get_configuration;

struct TestApp {
    pub addr: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() -> () {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &app.addr))
        .send()
        .await
        .expect("Failed to execute request to health_check.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> () {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=carlos%20jose&email=carlos.cruz%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app.addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request to /subscriptions.");

    // Asserts
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!("carlos.cruz@gmail.com", saved.email);
    assert_eq!("carlos jose", saved.name);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> () {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=carlos.cruz1500%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .get(format!("{}/subscriptions", &app.addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request to /subscriptions");

        // Asserts
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

async fn spawn_app() -> TestApp {
    // Port 0 is special-cased at the OS level, when trying to bind port 0
    // a scan will be triggered to find an available port, and the bind to it.
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let config = get_configuration().expect("Failed to get configuration.");
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to create connection pool.");

    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    TestApp {
        addr: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}
