use serde::{Deserialize, Serialize};

pub type Snowflake = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub roles: Option<Vec<Snowflake>>,
    pub joined_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Snowflake,
    pub name: String,
    pub color: Option<u32>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: Snowflake,
    pub name: Option<String>, 
    pub icon: Option<String>,
    pub owner_id: Option<Snowflake>,
    pub description: Option<String>,
    #[serde(default)]
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
    pub unavailable: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelType {
    Text = 0,
    Voice = 2,
    Category = 4,
}

impl From<u8> for ChannelType {
    fn from(value: u8) -> Self {
        match value {
            0 => ChannelType::Text,
            2 => ChannelType::Voice,
            4 => ChannelType::Category,
            _ => ChannelType::Text,
        }
    }
}

impl From<ChannelType> for u8 {
    fn from(ct: ChannelType) -> u8 {
        ct as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: ChannelType, 
    pub guild_id: Option<Snowflake>,
    pub name: Option<String>,
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub author: User,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub mentions: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ready {
    #[serde(rename = "version")]
    pub v: u64, 
    pub user: User,
    pub session_id: String,
    #[serde(default)]
    pub guilds: Vec<UnavailableGuild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayBotResponse {
    pub url: String,
    pub shards: Option<u32>, 
}