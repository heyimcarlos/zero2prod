use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, TestApp};

#[tokio::test]
async fn newsletter_are_not_delivered_to_unconfirmed_susbcribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter Title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": " <p>Newsletter body as html</p>"
        }
    });
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.addr))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // `Mock` asserts on drop that we haven't sent the newsletter email
}

async fn create_unconfirmed_subscriber(app: &TestApp) {
    let body = "name=carlos%20jose&email=carlos%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
}
