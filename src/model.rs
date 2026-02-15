use serde::{Deserialize, Serialize};

pub type Snowflake = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub bot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub author: User,
    pub content: String,
}

#[derive(Deserialize)]
pub struct GatewayBotResponse {
    pub url: String,
}