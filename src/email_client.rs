use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    pub client: Client,
    pub http_client: Client,
    pub base_url: String,
    pub sender: SubscriberEmail,
    pub auth_token: Secret<String>,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: Secret<String>) -> Self {
        Self {
            client: Client::new(),
            http_client: Client::new(),
            base_url,
            sender,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
