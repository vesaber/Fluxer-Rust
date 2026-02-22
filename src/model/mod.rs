pub mod voice;
use serde::{Deserialize, Serialize};

pub type Snowflake = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Snowflake,
    #[serde(default)]
    pub username: String,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    #[serde(default)]
    pub bot: Option<bool>,
    #[serde(default)]
    pub system: Option<bool>,
    pub public_flags: Option<u64>,
    pub premium: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: Snowflake,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub splash: Option<String>,
    pub owner_id: Option<Snowflake>,
    pub afk_channel_id: Option<Snowflake>,
    pub afk_timeout: Option<u64>,
    pub verification_level: Option<u64>,
    pub default_message_notifications: Option<u64>,
    pub explicit_content_filter: Option<u64>,
    pub roles: Option<Vec<Role>>,
    pub emojis: Option<Vec<Emoji>>,
    pub features: Option<Vec<String>>,
    pub member_count: Option<u64>,
    pub max_members: Option<u64>,
    pub description: Option<String>,
    pub preferred_locale: Option<String>,
    pub vanity_url_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<Snowflake>,
    pub joined_at: String,
    pub deaf: Option<bool>,
    pub mute: Option<bool>,
    pub pending: Option<bool>,
    pub permissions: Option<String>,
    pub communication_disabled_until: Option<String>,
}

impl Member {
    pub fn display_name(&self) -> &str {
        if let Some(nick) = &self.nick {
            return nick.as_str();
        }
        self.user
            .as_ref()
            .map(|u| u.username.as_str())
            .unwrap_or("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Snowflake,
    pub name: String,
    pub color: Option<u64>,
    pub hoist: Option<bool>,
    pub icon: Option<String>,
    pub position: Option<i64>,
    pub permissions: Option<String>,
    pub managed: Option<bool>,
    pub mentionable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emoji {
    pub id: Option<Snowflake>,
    pub name: Option<String>,
    pub roles: Option<Vec<Snowflake>>,
    pub user: Option<User>,
    pub require_colons: Option<bool>,
    pub managed: Option<bool>,
    pub animated: Option<bool>,
    pub available: Option<bool>,
}

impl Emoji {
    pub fn to_reaction_string(&self) -> String {
        match (&self.name, &self.id) {
            (Some(name), Some(id)) => format!("{}:{}", name, id),
            (Some(name), None) => name.clone(),
            _ => String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: Option<u8>,
    pub guild_id: Option<Snowflake>,
    pub position: Option<i64>,
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub nsfw: Option<bool>,
    pub last_message_id: Option<Snowflake>,
    pub bitrate: Option<u64>,
    pub user_limit: Option<u64>,
    pub rate_limit_per_user: Option<u64>,
    pub recipients: Option<Vec<User>>,
    pub icon: Option<String>,
    pub owner_id: Option<Snowflake>,
    pub parent_id: Option<Snowflake>,
    pub last_pin_timestamp: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelType {
    Text = 0,
    Dm = 1,
    Voice = 2,
    GroupDm = 3,
    Category = 4,
    Announcement = 5,
    Stage = 13,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverwrite {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: u8,
    pub allow: Option<String>,
    pub deny: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub author: User,
    pub member: Option<Member>,
    pub content: Option<String>,
    pub timestamp: Option<String>,
    pub edited_timestamp: Option<String>,
    pub tts: Option<bool>,
    pub mention_everyone: Option<bool>,
    pub mentions: Option<Vec<User>>,
    pub mention_roles: Option<Vec<Snowflake>>,
    pub attachments: Option<Vec<Attachment>>,
    pub embeds: Option<Vec<Embed>>,
    pub reactions: Option<Vec<Reaction>>,
    pub pinned: Option<bool>,
    pub webhook_id: Option<Snowflake>,
    #[serde(rename = "type")]
    pub kind: Option<u8>,
    pub referenced_message: Option<Box<Message>>,
    pub flags: Option<u64>,
    pub stickers: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedMessage {
    pub message: Message,
    pub pinned_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinsResponse {
    pub items: Vec<PinnedMessage>,
    pub has_more: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Snowflake,
    pub filename: Option<String>,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub url: Option<String>,
    pub proxy_url: Option<String>,
    pub height: Option<u64>,
    pub width: Option<u64>,
    pub ephemeral: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Embed {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub timestamp: Option<String>,
    pub color: Option<u64>,
    pub footer: Option<EmbedFooter>,
    pub image: Option<EmbedMedia>,
    pub thumbnail: Option<EmbedMedia>,
    pub video: Option<EmbedMedia>,
    pub author: Option<EmbedAuthor>,
    pub fields: Option<Vec<EmbedField>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedMedia {
    pub url: String,
    pub height: Option<u64>,
    pub width: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedAuthor {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub count: u64,
    pub me: bool,
    pub emoji: Emoji,
}

#[derive(Debug, Default)]
pub struct EmbedBuilder(Embed);

impl EmbedBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.0.title = Some(title.into());
        self
    }
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.0.description = Some(desc.into());
        self
    }
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }
    pub fn color(mut self, color: u64) -> Self {
        self.0.color = Some(color);
        self
    }
    pub fn timestamp(mut self, ts: impl Into<String>) -> Self {
        self.0.timestamp = Some(ts.into());
        self
    }
    pub fn footer(mut self, text: impl Into<String>, icon_url: Option<String>) -> Self {
        self.0.footer = Some(EmbedFooter { text: text.into(), icon_url });
        self
    }
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.0.image = Some(EmbedMedia { url: url.into(), height: None, width: None });
        self
    }
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.0.thumbnail = Some(EmbedMedia { url: url.into(), height: None, width: None });
        self
    }
    pub fn author(mut self, name: impl Into<String>, url: Option<String>, icon_url: Option<String>) -> Self {
        self.0.author = Some(EmbedAuthor { name: name.into(), url, icon_url });
        self
    }
    pub fn field(mut self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        let fields = self.0.fields.get_or_insert_with(Vec::new);
        fields.push(EmbedField { name: name.into(), value: value.into(), inline });
        self
    }
    pub fn build(self) -> Embed {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    pub code: String,
    pub guild: Option<PartialGuild>,
    pub channel: Option<PartialChannel>,
    pub inviter: Option<User>,
    pub target_user: Option<User>,
    pub approximate_member_count: Option<u64>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialGuild {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub splash: Option<String>,
    pub banner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialChannel {
    pub id: Snowflake,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: Option<u8>,
    pub guild_id: Option<Snowflake>,
    pub channel_id: Option<Snowflake>,
    pub user: Option<User>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub token: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStart {
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub user_id: Snowflake,
    pub timestamp: u64,
    pub member: Option<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionAdd {
    pub user_id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub message_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub member: Option<Member>,
    pub emoji: Emoji,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionRemove {
    pub user_id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub message_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub emoji: Emoji,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionRemoveAll {
    pub channel_id: Option<Snowflake>,
    pub message_id: Snowflake,
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionRemoveEmoji {
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub message_id: Snowflake,
    pub emoji: Emoji,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUpdate {
    pub id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub author: Option<User>,
    pub content: Option<String>,
    pub edited_timestamp: Option<String>,
    pub embeds: Option<Vec<Embed>>,
    pub attachments: Option<Vec<Attachment>>,
    pub pinned: Option<bool>,
    pub flags: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelete {
    pub id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleteBulk {
    pub ids: Vec<Snowflake>,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberAdd {
    pub guild_id: Snowflake,
    #[serde(flatten)]
    pub member: Member,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberRemove {
    pub guild_id: Snowflake,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberUpdate {
    pub guild_id: Snowflake,
    pub roles: Vec<Snowflake>,
    pub user: User,
    pub nick: Option<String>,
    pub joined_at: Option<String>,
    pub pending: Option<bool>,
    pub communication_disabled_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ban {
    pub reason: Option<String>,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildBanAdd {
    pub guild_id: Snowflake,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildBanRemove {
    pub guild_id: Snowflake,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRoleCreate {
    pub guild_id: Snowflake,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRoleUpdate {
    pub guild_id: Snowflake,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRoleDelete {
    pub guild_id: Snowflake,
    pub role_id: Snowflake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDelete {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: Option<u8>,
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPinsUpdate {
    pub guild_id: Option<Snowflake>,
    pub channel_id: Option<Snowflake>,
    pub last_pin_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ready {
    pub v: Option<u64>,
    pub session_id: String,
    pub resume_gateway_url: Option<String>,
    pub user: User,
    pub guilds: Option<Vec<UnavailableGuild>>,
    pub shard: Option<[u64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
    pub unavailable: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GatewayBotResponse {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildEmojisUpdate {
    pub guild_id: Snowflake,
    pub emojis: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildStickersUpdate {
    pub guild_id: Snowflake,
    pub stickers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRoleUpdateBulk {
    pub guild_id: Snowflake,
    pub roles: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUpdateBulk {
    pub guild_id: Option<Snowflake>,
    pub channels: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCreate {
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteDelete {
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhooksUpdate {
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MessageCreatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_message_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageReference {
    pub message_id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_if_not_exists: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ChannelCreatePayload {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EditMemberPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Snowflake>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Option<Snowflake>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub communication_disabled_until: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateRolePayload {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentionable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EditRolePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentionable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateInvitePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct GetMessagesQuery {
    pub limit: Option<u8>,
    pub before: Option<Snowflake>,
    pub after: Option<Snowflake>,
    pub around: Option<Snowflake>,
}

impl GetMessagesQuery {
    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(l) = self.limit {
            parts.push(format!("limit={}", l.min(100)));
        }
        if let Some(ref b) = self.before {
            parts.push(format!("before={}", b));
        }
        if let Some(ref a) = self.after {
            parts.push(format!("after={}", a));
        }
        if let Some(ref ar) = self.around {
            parts.push(format!("around={}", ar));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EditGuildPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afk_channel_id: Option<Option<Snowflake>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afk_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_level: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_message_notifications: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_content_filter: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct WebhookExecutePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
}