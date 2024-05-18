use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/subscriptions/confirm", &app.addr))
        .send()
        .await
        .expect("Failed to execute request to `/subscriptions/confirm`");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=carlos%20jose&email=carlos.cruz%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.to_owned()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    let response = reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute request to `/subscriptions/confirm`");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // clicking on the confirmation link sends a GET to `/subscriptions/confirm` which
    // should confirm a subscriber if the subscription token is available.
    // Arrange
    let app = spawn_app().await;
    let body = "name=carlos%20jose&email=carlos.cruz%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.to_owned()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Click the link
    reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute request to `/subscriptions/confirm`")
        .error_for_status()
        .unwrap();

    // Assert
    let saved = sqlx::query!("SELECT name, email, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to read the subscriptions table");

    assert_eq!(saved.name, "carlos");
    assert_eq!(saved.email, "carlos.cruz@gmail.com");
    assert_eq!(saved.status, "confirmed");
}
