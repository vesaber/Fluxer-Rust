//! Typed structs for everything the Fluxer API returns -- users, guilds,
//! channels, messages, embeds, and all the gateway event payloads.
//!
//! Most fields are `Option<T>` because the API doesn't always include
//! everything depending on the endpoint.

pub mod voice;
use serde::{Deserialize, Serialize};

/// All entity IDs in the Fluxer API are snowflake strings.
pub type Snowflake = String;

/// A Fluxer user account. Both bot and human accounts share this struct.
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

/// A guild (server). Full data arrives via `GUILD_CREATE`; [`UnavailableGuild`] is used before then.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: Snowflake,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub splash: Option<String>,
    pub owner_id: Option<Snowflake>,
    pub afk_channel_id: Option<Snowflake>,
    /// AFK timeout in seconds.
    pub afk_timeout: Option<u64>,
    pub verification_level: Option<u64>,
    /// 0 = all messages, 1 = only mentions.
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

/// A guild member. Wraps a [`User`] with guild-specific info like nickname and roles.
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
    /// ISO 8601 timestamp. If set, the member is timed out until then.
    pub communication_disabled_until: Option<String>,
}

impl Member {
    /// Returns the member's nickname if they have one, otherwise their username.
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

/// A guild role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Snowflake,
    pub name: String,
    /// Integer color value. 0 means no color.
    pub color: Option<u64>,
    /// Whether this role shows up separately in the member list.
    pub hoist: Option<bool>,
    pub icon: Option<String>,
    pub position: Option<i64>,
    /// Permission bitfield.
    pub permissions: Option<String>,
    /// Managed roles are created by integrations (bot roles, booster role, etc).
    pub managed: Option<bool>,
    pub mentionable: Option<bool>,
}

/// Custom or standard emoji. Custom emojis have both `name` and `id`,
/// standard unicode emojis only have `name`.
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
    /// Formats the emoji for use in reaction endpoints. Returns `"name:id"` for
    /// custom emojis or just the name for unicode ones.
    pub fn to_reaction_string(&self) -> String {
        match (&self.name, &self.id) {
            (Some(name), Some(id)) => format!("{}:{}", name, id),
            (Some(name), None) => name.clone(),
            _ => String::new(),
        }
    }
}

/// A channel — text, voice, DM, category, etc. Check `kind` to tell them apart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Snowflake,
    /// Channel type, see [`ChannelType`] for values.
    #[serde(rename = "type")]
    pub kind: Option<u64>,
    pub guild_id: Option<Snowflake>,
    pub position: Option<i64>,
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub nsfw: Option<bool>,
    pub last_message_id: Option<Snowflake>,
    /// Bits per second, for voice channels.
    pub bitrate: Option<u64>,
    /// 0 = unlimited.
    pub user_limit: Option<u64>,
    /// Slowmode, in seconds.
    pub rate_limit_per_user: Option<u64>,
    /// Recipients for DM/group DM channels.
    pub recipients: Option<Vec<User>>,
    pub icon: Option<String>,
    pub owner_id: Option<Snowflake>,
    pub parent_id: Option<Snowflake>,
    pub last_pin_timestamp: Option<String>,
}

/// Numeric channel type values that map to [`Channel::kind`].
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

/// Permission overwrite for a channel. `kind` is 0 for role, 1 for member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverwrite {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: u8,
    pub allow: Option<String>,
    pub deny: Option<String>,
}

/// A message sent in a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub author: User,
    /// Only present in guild messages.
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
    /// 0 = default, 19 = reply, etc.
    #[serde(rename = "type")]
    pub kind: Option<u8>,
    /// Full message object for the message being replied to, if the server includes it.
    pub referenced_message: Option<Box<Message>>,
    /// Reference metadata (message/channel/guild IDs) always present on reply messages.
    pub message_reference: Option<MessageReference>,
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

/// A file attached to a message.
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

/// A file to attach when sending a message. Pass one or more of these to
/// [`Http::send_files`] or [`Http::send_message_with_files`].
pub struct AttachmentFile {
    /// Filename shown in the client, e.g. `"image.png"`.
    pub filename: String,
    pub data: Vec<u8>,
    /// MIME type, e.g. `"image/png"`. If `None`, defaults to `"application/octet-stream"`.
    pub content_type: Option<String>,
}

/// Rich embed. Use [`EmbedBuilder`] to construct these.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Embed {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub timestamp: Option<String>,
    /// Sidebar color as an integer, e.g. `0xFF0000` for red.
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
    /// If true, fields can sit side-by-side.
    #[serde(default)]
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub count: u64,
    pub me: bool,
    pub emoji: Emoji,
}

/// Builder for [`Embed`]. Chain methods and call `.build()` at the end.
///
/// ```rust
/// use fluxer::prelude::*;
///
/// let embed = EmbedBuilder::new()
///     .title("Hello")
///     .description("World")
///     .color(0x00FF00)
///     .build();
/// ```
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

/// A webhook. Use [`Http::execute_webhook`](crate::http::Http::execute_webhook) to post through one.
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

// --- Gateway event payloads ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStart {
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub user_id: Snowflake,
    /// Unix timestamp in seconds.
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

/// Partial message data from an edit. Only changed fields are populated.
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

/// Received once after connecting. Contains the bot user, session info for
/// resuming, and the initial guild list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ready {
    pub v: Option<u64>,
    pub session_id: String,
    pub resume_gateway_url: Option<String>,
    pub user: User,
    /// Guilds come in as unavailable first; full data arrives via GUILD_CREATE events.
    pub guilds: Option<Vec<UnavailableGuild>>,
    /// `[shard_id, num_shards]`
    pub shard: Option<[u64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
    /// `true` = temporary outage, `false`/absent = bot was removed.
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

/// Full guild state sync, sent in response to an op 14 subscription request.
/// Contains the same data as GUILD_CREATE but for already-known guilds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildSync {
    pub id: Snowflake,
    pub roles: Option<Vec<Role>>,
    pub channels: Option<Vec<Channel>>,
    pub emojis: Option<Vec<Emoji>>,
    pub stickers: Option<Vec<serde_json::Value>>,
    pub members: Option<Vec<Member>>,
    pub member_count: Option<u64>,
    pub online_count: Option<u64>,
    pub presences: Option<Vec<serde_json::Value>>,
    pub voice_states: Option<Vec<serde_json::Value>>,
    pub joined_at: Option<String>,
    pub unavailable: Option<bool>,
}

/// Custom status set by a user (the emoji + text shown under their name).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatus {
    pub text: Option<String>,
    pub emoji_id: Option<Snowflake>,
    pub emoji_name: Option<String>,
    pub expires_at: Option<String>,
}

/// Fired when a user's presence (status/activity) changes in a guild.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub user: User,
    pub guild_id: Option<Snowflake>,
    /// `"online"`, `"idle"`, `"dnd"`, or `"offline"`.
    pub status: Option<String>,
    pub mobile: Option<bool>,
    pub afk: Option<bool>,
    pub custom_status: Option<CustomStatus>,
    pub activities: Option<Vec<serde_json::Value>>,
    pub client_status: Option<serde_json::Value>,
}

/// Multiple presence updates batched together. `guild_id` is set when the
/// presences all belong to the same guild.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateBulk {
    pub presences: Vec<PresenceUpdate>,
    pub guild_id: Option<Snowflake>,
}

/// Fired when the current user's settings change. Structure varies; use
/// the raw `data` field to access specific setting keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsUpdate {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Partial user fields sent with USER_UPDATE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdate {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
    pub flags: Option<u64>,
}

/// Fired when a channel is marked as read from another session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAck {
    pub channel_id: Option<Snowflake>,
    pub message_id: Option<Snowflake>,
}

/// A single session entry inside [`SessionsReplace`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub session_id: String,
    /// `"online"`, `"idle"`, `"dnd"`, `"invisible"`, or `"offline"`.
    pub status: Option<String>,
    pub mobile: Option<bool>,
    pub afk: Option<bool>,
}

/// Replaces all active sessions for the current user. Sent when another
/// device connects or changes presence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsReplace(pub Vec<SessionEntry>);

/// Relationship types from the Fluxer source (`RelationshipTypes` enum).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    Friend = 1,
    Blocked = 2,
    IncomingRequest = 3,
    OutgoingRequest = 4,
}

/// Fired when a relationship is added (friend accepted, request sent, block added, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAdd {
    /// The other user's ID.
    pub id: Snowflake,
    /// See [`RelationshipType`] for values.
    #[serde(rename = "type")]
    pub kind: u8,
}

/// Fired when an existing relationship changes type (e.g. pending → friend).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipUpdate {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: u8,
}

/// Fired when a relationship is removed (unfriend, unblock, request cancelled).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRemove {
    pub id: Snowflake,
}

/// Voice state for a participant inside a DM/group call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallVoiceState {
    pub user_id: Snowflake,
    pub channel_id: Option<Snowflake>,
    pub session_id: Option<String>,
    pub self_mute: Option<bool>,
    pub self_deaf: Option<bool>,
    pub self_video: Option<bool>,
    pub self_stream: Option<bool>,
}

/// A VC was started in a DM or group DM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallCreate {
    pub channel_id: Snowflake,
    pub message_id: Option<Snowflake>,
    pub region: Option<String>,
    pub voice_states: Option<Vec<CallVoiceState>>,
    /// User IDs of people currently being rung.
    pub ringing: Option<Vec<Snowflake>>,
}

/// A call's state changed (someone joined/left, region changed, etc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallUpdate {
    pub channel_id: Snowflake,
    pub message_id: Option<Snowflake>,
    pub region: Option<String>,
    pub voice_states: Option<Vec<CallVoiceState>>,
    pub ringing: Option<Vec<Snowflake>>,
}

/// A VC ended or was declined.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallDelete {
    pub channel_id: Snowflake,
}

/// A user was added to a group DM channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRecipientAdd {
    pub channel_id: Snowflake,
    pub user: User,
}

/// A user was removed from a group DM channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRecipientRemove {
    pub channel_id: Snowflake,
    pub user: User,
}

/// A batch of reactions added to a single message (server side debounce).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionAddMany {
    pub channel_id: Option<Snowflake>,
    pub message_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    /// Each entry has `user_id`, `emoji` (`id`/`name`), and optionally `member`.
    pub reactions: Vec<serde_json::Value>,
}

/// Lazy passive guild update: channel last-message IDs, voice states, and
/// channel create/update/delete diffs. Sent for large guilds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveUpdates {
    pub guild_id: Snowflake,
    /// Map of `channel_id → last_message_id`.
    pub channels: Option<serde_json::Value>,
    pub voice_states: Option<Vec<serde_json::Value>>,
    pub created_channels: Option<Vec<Channel>>,
    pub updated_channels: Option<Vec<Channel>>,
    pub deleted_channel_ids: Option<Vec<Snowflake>>,
}

/// An op inside a [`GuildMemberListUpdate`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberListOp {
    /// `"SYNC"`, `"INSERT"`, `"UPDATE"`, `"DELETE"`, or `"INVALIDATE"`.
    pub op: String,
    /// `[start, end]` range, present for SYNC and INVALIDATE.
    pub range: Option<[u64; 2]>,
    /// Item index, present for INSERT / UPDATE / DELETE.
    pub index: Option<u64>,
    /// Batch of items for SYNC.
    pub items: Option<Vec<serde_json::Value>>,
    /// Single item for INSERT / UPDATE / DELETE.
    pub item: Option<serde_json::Value>,
}

/// Incremental member list update for a guild channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberListUpdate {
    pub guild_id: Snowflake,
    /// List identifier (usually matches `channel_id`).
    pub id: String,
    pub channel_id: Option<Snowflake>,
    pub member_count: Option<u64>,
    pub online_count: Option<u64>,
    pub groups: Option<Vec<serde_json::Value>>,
    pub ops: Vec<MemberListOp>,
}

/// Response to REQUEST_GUILD_MEMBERS (op 8). Contains a chunk of guild members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMembersChunk {
    pub guild_id: Snowflake,
    pub members: Vec<Member>,
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub not_found: Option<Vec<Snowflake>>,
    pub presences: Option<Vec<serde_json::Value>>,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceServerUpdate {
    pub token: String,
    pub guild_id: Option<Snowflake>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resumed {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPinsAck {
    pub channel_id: Snowflake,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPinnedDmsUpdate {
    pub pinned: Vec<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNoteUpdate {
    pub id: Snowflake,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnectionsUpdate {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGuildSettingsUpdate {
    pub guild_id: Option<Snowflake>,
    #[serde(flatten)]
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionChange {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMessageCreate {
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMessageDelete {
    pub message_id: Snowflake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentMentionDelete {
    pub message_id: Snowflake,
}

// --- Request payloads ---

/// Attachment metadata entry for the `attachments` field of [`MessageCreatePayload`].
/// `id` must match the index used in the corresponding `files[N]` multipart part.
#[derive(Debug, Clone, Serialize)]
pub struct AttachmentMetadata {
    pub id: u64,
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Payload for sending/editing messages. All fields optional; only set what you need.
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
    /// Required when uploading files. Populated automatically by
    /// [`Http::send_message_with_files`]; you don't need to set this manually.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<AttachmentMetadata>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// For nullable fields like `nick`, use `Some(None)` to clear them.
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
    /// Seconds. 0 = never expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
    /// 0 = unlimited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    /// If true, members joined via this invite get kicked when they disconnect
    /// (unless they've been given a role).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
}

/// Query params for fetching messages. Only set one of `before`/`after`/`around`.
#[derive(Debug, Clone, Default)]
pub struct GetMessagesQuery {
    /// 1-100.
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

/// Query payload for [`Http::search_messages`]. All fields are optional — set
/// only the filters you need. `scope` defaults to `"current"` server side.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SearchMessagesQuery {
    /// Results per page, 1–25.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hits_per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Full-text content search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Restrict to these channel IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Vec<Snowflake>>,
    /// Restrict to messages by these author IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Vec<Snowflake>>,
    /// Filter by attachment/embed type: `"image"`, `"video"`, `"file"`,
    /// `"sticker"`, `"embed"`, `"link"`, `"poll"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    /// `"timestamp"` or `"relevance"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    /// `"asc"` or `"desc"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
    /// `"current"`, `"open_dms"`, `"all_dms"`, `"all_guilds"`, or `"all"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Returned by [`Http::search_messages`].
#[derive(Debug, Clone, Deserialize)]
pub struct SearchMessagesResponse {
    pub messages: Vec<Message>,
    pub total: u64,
    pub hits_per_page: u8,
    pub page: u32,
}

/// A single entry for [`Http::ack_bulk`]. Marks `message_id` as the last-read
/// message in `channel_id`.
#[derive(Debug, Clone, Serialize)]
pub struct ReadStateAck {
    pub channel_id: Snowflake,
    pub message_id: Snowflake,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct WebhookExecutePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Override the webhook's name for this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Override the webhook's avatar for this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
}