use linkify::LinkFinder;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> () {
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
    let response = app.post_subscriptions(body.into()).await;

    // Asserts
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_a_new_subscriber() -> () {
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
    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT name, email, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    // Asserts
    assert_eq!(saved.email, "carlos.cruz@gmail.com");
    assert_eq!(saved.name, "carlos jose");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> () {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=carlos.cruz1500%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Asserts
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() -> () {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=le%20guin%40gmail.com", "empty name"),
        ("name=carlos%20cruz&email=", "empty email"),
        ("name=Carlos&email=not-an-email", "invalid email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
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
    app.post_subscriptions(body.into()).await;

    // Assert
    // Mock assert on drop
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=carlos%20jose&email=carlos.cruz%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    let received_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value =
        serde_json::from_slice(&received_request.body).expect("Failed to parse body");
    let get_link = |s: &str| -> String {
        let finder = LinkFinder::new();
        let links: Vec<_> = finder
            .links(s)
            .filter(|link| *link.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let text_link = get_link(body["TextBody"].as_str().unwrap());
    let html_link = get_link(body["HtmlBody"].as_str().unwrap());
    assert_eq!(text_link, html_link);
}
