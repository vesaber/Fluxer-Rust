use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use crate::model::*;
use serde_json::json;

pub struct Http {
    client: reqwest::Client,
    pub base_url: String,
    token: String,
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
            token: token.to_string(),
        }
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub async fn get_gateway(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/gateway/bot", self.base_url);
        let res: GatewayBotResponse = self.client.get(&url).send().await?.json().await?;
        Ok(res.url)
    }

    pub async fn get_me(&self) -> Result<User, reqwest::Error> {
        let url = format!("{}/users/@me", self.base_url);
        self.client.get(&url).send().await?.json().await
    }

    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<Message, reqwest::Error> {
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);
        let body = json!({ "content": content });
        self.client.post(&url).json(&body).send().await?.json().await
    }

    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/channels/{}/messages/{}", self.base_url, channel_id, message_id);
        self.client.delete(&url).send().await?;
        Ok(())
    }

    pub async fn get_guild(&self, guild_id: &str) -> Result<Guild, reqwest::Error> {
        let url = format!("{}/guilds/{}", self.base_url, guild_id);
        self.client.get(&url).send().await?.json().await
    }

    pub async fn create_channel(&self, guild_id: &str, name: &str, kind: ChannelType) -> Result<Channel, reqwest::Error> {
        let url = format!("{}/guilds/{}/channels", self.base_url, guild_id);
        let body = json!({ "name": name, "type": kind as u8 });

        let response = self.client.post(&url).json(&body).send().await?;
        response.json().await
    }

    pub async fn ban_member(&self, guild_id: &str, user_id: &str, reason: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/guilds/{}/bans/{}", self.base_url, guild_id, user_id);
        let body = json!({ "reason": reason });
        self.client.put(&url).json(&body).send().await?;
        Ok(())
    }
}