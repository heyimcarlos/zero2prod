use crate::helpers::spawn_app;

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
