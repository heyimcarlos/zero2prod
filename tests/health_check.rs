use actix_web::web::UrlEncoded;

#[tokio::test]
async fn health_check_works() -> () {
    // Arrange
    let addr = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", &addr))
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
    let app_addr = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = "name=carlos%20jose&email=carlos.cruz%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app_addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request to /subscriptions.");

    // Asserts
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> () {
    // Arrange
    let app_addr = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=carlos.cruz1500%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(app_addr.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request to /subscriptions");

        // Asserts
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API failed with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

fn spawn_app() -> String {
    // Port 0 is special-cased at the OS level, when trying to bind port 0
    // a scan will be triggered to find an available port, and the bind to it.
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address.");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
