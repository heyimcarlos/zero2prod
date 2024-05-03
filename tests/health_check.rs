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
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
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
