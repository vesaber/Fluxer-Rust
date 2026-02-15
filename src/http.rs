use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use crate::model::{Message, GatewayBotResponse};
use serde_json::json;

pub struct Http {
    client: reqwest::Client,
    pub base_url: String,
}

impl Http {
    pub fn new(token: &str, base_url: String) -> Self {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bot {}", token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url,
        }
    }

    pub async fn get_gateway(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/gateway/bot", self.base_url);
        let text = self.client.get(&url).send().await?.text().await?;
        let response: GatewayBotResponse = serde_json::from_str(&text)?;
        Ok(response.url)
    }

    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<Message, reqwest::Error> {
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);
        let body = json!({ "content": content });
        self.client.post(&url).json(&body).send().await?.json().await
    }
}