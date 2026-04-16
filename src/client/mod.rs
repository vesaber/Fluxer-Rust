//! Gateway client and connection management.

use std::collections::HashMap;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use crate::cache::Cache;
use crate::error::ClientError;
use crate::event::EventHandler;
use crate::http::Http;
use crate::model::voice::VoiceState;
use std::time::Duration;

const DEFAULT_API_URL: &str = "https://api.fluxer.app/v1";
const DEFAULT_GATEWAY_URL: &str = "wss://gateway.fluxer.app/?v=1&encoding=json";

enum LoopControl {
    Done,
    Reconnect { resume: bool },
}

/// Shared state passed into every event handler call. This is how you interact
/// with the API from inside your event handlers.
///
/// ```rust,no_run
/// # use fluxer::prelude::*;
/// # async fn example(ctx: Context) {
/// ctx.http.send_message("channel_id", "Hello!").await.unwrap();
/// # }
/// ```
#[derive(Clone)]
pub struct Context {
    /// HTTP client for REST API calls.
    pub http: Arc<Http>,
    /// In-memory cache populated automatically from gateway events.
    pub cache: Arc<Cache>,
    /// Raw gateway sender. You probably won't need this directly --
    /// voice join/leave use it internally.
    pub gateway_tx: Arc<tokio::sync::mpsc::Sender<String>>,
    /// Per-guild voice state, populated from `VOICE_STATE_UPDATE` and `VOICE_SERVER_UPDATE`.
    /// Used internally by `join_voice` / `leave_voice`.
    pub voice_states: Arc<Mutex<HashMap<String, VoiceState>>>,
    pub(crate) live_rooms: Arc<Mutex<HashMap<String, std::sync::Arc<livekit::Room>>>>,
    pub(crate) handler: Arc<dyn EventHandler>,
}

impl Context {
    /// Joins a voice channel. Sends an opcode 4 to the gateway and waits
    /// up to 10 seconds for the server to send back connection details.
    pub async fn join_voice(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<crate::voice::FluxerVoiceConnection, ClientError> {
        {
            let mut states = self.voice_states.lock().await;
            states.remove(guild_id);
        }

        let join_payload = serde_json::json!({
            "op": 4,
            "d": {
                "guild_id": guild_id,
                "channel_id": channel_id,
                "self_mute": false,
                "self_deaf": false
            }
        });
        self.gateway_tx
            .send(join_payload.to_string())
            .await
            .map_err(|e| ClientError::Voice(e.to_string()))?;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        let voice_state = loop {
            {
                let states = self.voice_states.lock().await;
                if let Some(vs) = states.get(guild_id) {
                    if !vs.token.is_empty() && !vs.endpoint.is_empty() {
                        break vs.clone();
                    }
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(ClientError::Voice(
                    "Timed out waiting for VOICE_SERVER_UPDATE".into(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        let conn = crate::voice::FluxerVoiceConnection::connect(
            &voice_state.endpoint,
            &voice_state.token,
            self.clone(),
        )
        .await
        .map_err(|e| ClientError::Voice(e.to_string()))?;

        self.live_rooms.lock().await.insert(guild_id.to_string(), conn.room.clone());

        Ok(conn)
    }

    /// Leaves a voice channel. Closes the LiveKit room and tells the gateway.
    pub async fn leave_voice(&self, guild_id: &str) -> Result<(), ClientError> {
        if let Some(room) = self.live_rooms.lock().await.remove(guild_id) {
            let _ = room.close().await;
        }

        let payload = serde_json::json!({
            "op": 4,
            "d": {
                "guild_id": guild_id,
                "channel_id": null,
                "self_mute": false,
                "self_deaf": false
            }
        });
        self.gateway_tx
            .send(payload.to_string())
            .await
            .map_err(|e| ClientError::Voice(e.to_string()))?;
        self.voice_states.lock().await.remove(guild_id);
        Ok(())
    }

    /// Subscribes to a guild's events (op 14). Called automatically for all
    /// guilds on READY, but you can call this manually if you join a new guild
    /// after the initial connection (e.g. from inside `on_guild_create`).
    pub async fn subscribe_guild(&self, guild_id: &str) -> Result<(), ClientError> {
        let payload = serde_json::json!({
            "op": 14,
            "d": {
                "subscriptions": {
                    guild_id: { "active": true, "typing": true }
                }
            }
        });
        self.gateway_tx
            .send(payload.to_string())
            .await
            .map_err(|e| ClientError::Voice(e.to_string()))
    }
}

/// Builder for creating a [`Client`]. You need at minimum a token and an event handler.
///
/// ```rust,no_run
/// use fluxer::prelude::*;
/// # struct MyHandler;
/// # #[async_trait::async_trait]
/// # impl EventHandler for MyHandler {}
///
/// let client = Client::builder("token")
///     .event_handler(MyHandler)
///     .build();
/// ```
pub struct ClientBuilder {
    token: String,
    api_url: String,
    handler: Option<Arc<dyn EventHandler>>,
    user_token: bool,
}

impl ClientBuilder {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_url: DEFAULT_API_URL.to_string(),
            handler: None,
            user_token: false,
        }
    }

    /// Use a user token instead of a bot token (no `Bot ` prefix on HTTP requests).
    pub fn user_token(mut self) -> Self {
        self.user_token = true;
        self
    }

    /// Sets the event handler. Required -- the builder panics at `.build()` without this.
    pub fn event_handler(mut self, handler: impl EventHandler + 'static) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }

    /// Override the API base URL. Defaults to `https://api.fluxer.app/v1`.
    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    pub fn build(self) -> Client {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = if self.user_token {
            Arc::new(Http::new_user(&self.token, self.api_url))
        } else {
            Arc::new(Http::new(&self.token, self.api_url))
        };
        Client {
            http,
            cache: Cache::new(),
            handler: self.handler.expect("call .event_handler() before .build()"),
        }
    }
}

/// The gateway client. Manages the WebSocket connection, heartbeating,
/// reconnection, and event dispatch.
///
/// Call [`start`](Client::start) to connect. It runs until a fatal error
/// happens (like an invalid token) and reconnects automatically on transient
/// failures.
pub struct Client {
    pub(crate) http: Arc<Http>,
    cache: Arc<Cache>,
    handler: Arc<dyn EventHandler>,
}

impl Client {
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(token)
    }

    /// Connects to the gateway and starts processing events. Blocks forever
    /// unless a fatal error occurs.
    pub async fn start(&mut self) -> Result<(), ClientError> {
        let mut session_id: Option<String> = None;
        let mut resume_url: Option<String> = None;
        let mut last_seq: Option<u64> = None;
        let mut backoff = Duration::from_secs(1);

        loop {
            let result = self
                .run_session(&mut session_id, &mut resume_url, &mut last_seq)
                .await;

            match result {
                Ok(LoopControl::Done) => return Ok(()),

                Ok(LoopControl::Reconnect { resume }) => {
                    if !resume {
                        session_id = None;
                        resume_url = None;
                        last_seq = None;
                        let jitter = Duration::from_millis(1000 + (rand::random::<u64>() % 4000));
                        tokio::time::sleep(jitter).await;
                    } else {
                        eprintln!("[fluxer-rs] Reconnecting in {:?} (will resume)...", backoff);
                        tokio::time::sleep(backoff).await;
                        backoff = (backoff * 2).min(Duration::from_secs(60));
                        continue;
                    }
                }

                Err(ClientError::ConnectionClosed) => {
                    eprintln!("[fluxer-rs] Connection closed, reconnecting in {:?}...", backoff);
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(Duration::from_secs(60));
                    continue;
                }

                Err(e) => return Err(e),
            }

            backoff = Duration::from_secs(1);
        }
    }

    async fn run_session(
        &self,
        session_id: &mut Option<String>,
        resume_url: &mut Option<String>,
        last_seq: &mut Option<u64>,
    ) -> Result<LoopControl, ClientError> {
        let gateway_url = if session_id.is_some() {
            resume_url
                .clone()
                .unwrap_or_else(|| DEFAULT_GATEWAY_URL.to_string())
        } else {
            match self.http.get_gateway().await {
                Ok(url) => {
                    let base = url.trim_end_matches('/');
                    format!("{}/?v=1&encoding=json", base)
                }
                Err(_) => DEFAULT_GATEWAY_URL.to_string(),
            }
        };

        let (ws_stream, _) = connect_async(&gateway_url).await?;
        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));
        let seq_shared: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(*last_seq));
        let ack_shared: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
        let (gateway_tx, mut gateway_rx) = tokio::sync::mpsc::channel::<String>(64);
        {
            let write_fwd = write.clone();
            tokio::spawn(async move {
                while let Some(msg) = gateway_rx.recv().await {
                    let mut guard = write_fwd.lock().await;
                    if guard
                        .send(WsMessage::Text(msg.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            });
        }

        let ctx = Context {
            http: self.http.clone(),
            cache: self.cache.clone(),
            gateway_tx: Arc::new(gateway_tx),
            voice_states: Arc::new(Mutex::new(HashMap::new())),
            live_rooms: Arc::new(Mutex::new(HashMap::new())),
            handler: self.handler.clone(),
        };

        let token = self.http.get_token().to_string();
        if let (Some(sid), Some(seq)) = (session_id.as_deref(), *last_seq) {
            let resume_payload = serde_json::json!({
                "op": 6,
                "d": { "token": token, "session_id": sid, "seq": seq }
            });
            write
                .lock()
                .await
                .send(WsMessage::Text(resume_payload.to_string().into()))
                .await?;
        } else {
            let identify = serde_json::json!({
                "op": 2,
                "d": {
                    "token": token,
                    "intents": 0,   // Fluxer has no intents yet
                    "properties": {
                        "os": "linux",
                        "browser": "fluxer-rust",
                        "device": "fluxer-rust"
                    }
                }
            });
            write
                .lock()
                .await
                .send(WsMessage::Text(identify.to_string().into()))
                .await?;
        }

        let handler = self.handler.clone();

        while let Some(msg_result) = read.next().await {
            let text = match msg_result? {
                WsMessage::Text(t) => t,
                WsMessage::Close(frame) => {
                    let code = frame.as_ref().map(|f| u16::from(f.code)).unwrap_or(0);
                    match code {
                        4004 => {
                            eprintln!("[fluxer-rs] Authentication failed (4004) — invalid token, shutting down.");
                            return Ok(LoopControl::Done);
                        }
                        4010 => {
                            eprintln!("[fluxer-rs] Invalid shard (4010) — shutting down.");
                            return Ok(LoopControl::Done);
                        }
                        4011 => {
                            eprintln!("[fluxer-rs] Sharding required (4011) — shutting down.");
                            return Ok(LoopControl::Done);
                        }
                        4012 => {
                            eprintln!("[fluxer-rs] Invalid API version (4012) — shutting down.");
                            return Ok(LoopControl::Done);
                        }
                        _ => return Err(ClientError::ConnectionClosed),
                    }
                }
                WsMessage::Ping(d) => {
                    let _ = write.lock().await.send(WsMessage::Pong(d)).await;
                    continue;
                }
                _ => continue,
            };

            let payload: Value = serde_json::from_str(text.as_str())?;
            let op = payload["op"].as_u64().unwrap_or(255);

            if let Some(s) = payload["s"].as_u64() {
                *last_seq = Some(s);
                *seq_shared.lock().await = Some(s);
            }

            match op {
                10 => {
                    let interval_ms = payload["d"]["heartbeat_interval"]
                        .as_u64()
                        .unwrap_or(41_250);

                    let write_hb = write.clone();
                    let seq_hb = seq_shared.clone();
                    let ack_hb = ack_shared.clone();

                    tokio::spawn(async move {
                        let jitter = Duration::from_millis(
                            (rand::random::<u64>() % interval_ms).max(1),
                        );
                        tokio::time::sleep(jitter).await;

                        let mut ticker =
                            tokio::time::interval(Duration::from_millis(interval_ms));
                        loop {
                            ticker.tick().await;

                            {
                                let mut ack = ack_hb.lock().await;
                                if !*ack {
                                    eprintln!(
                                        "[fluxer-rs] No heartbeat ACK — zombie connection, dropping."
                                    );
                                    break;
                                }
                                *ack = false;
                            }

                            let seq = *seq_hb.lock().await;
                            let hb = serde_json::json!({ "op": 1, "d": seq });
                            let mut guard = write_hb.lock().await;
                            if guard
                                .send(WsMessage::Text(hb.to_string().into()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    });
                }

                11 => {
                    *ack_shared.lock().await = true;
                }

                0 => {
                    let event_type = payload["t"].as_str().unwrap_or("").to_string();
                    let data = payload["d"].clone();
                    let ctx2 = ctx.clone();
                    let handler2 = handler.clone();

                    if event_type == "READY" {
                        if let Some(sid) = data["session_id"].as_str() {
                            *session_id = Some(sid.to_string());
                        }
                        if let Some(rurl) = data["resume_gateway_url"].as_str() {
                            *resume_url = Some(format!(
                                "{}/?v=1&encoding=json",
                                rurl.trim_end_matches('/')
                            ));
                        }
                        // Subscribe to all guilds (op 14) so events are received.
                        // Without this, self-bots only receive DM and system events.
                        if let Some(guilds) = data["guilds"].as_array() {
                            let mut subscriptions = serde_json::Map::new();
                            for g in guilds {
                                if let Some(id) = g["id"].as_str() {
                                    subscriptions.insert(
                                        id.to_string(),
                                        serde_json::json!({ "active": true, "typing": true }),
                                    );
                                }
                            }
                            if !subscriptions.is_empty() {
                                let lazy = serde_json::json!({
                                    "op": 14,
                                    "d": { "subscriptions": subscriptions }
                                });
                                let _ = ctx.gateway_tx.send(lazy.to_string()).await;
                            }
                        }
                    }

                    tokio::spawn(async move {
                        cache_update(&ctx2.cache, &event_type, &data).await;
                        dispatch_event(event_type, data, ctx2, handler2).await;
                    });
                }

                7 => {
                    eprintln!("[fluxer-rs] Received op 7 Reconnect.");
                    return Ok(LoopControl::Reconnect { resume: true });
                }

                9 => {
                    let resumable = payload["d"].as_bool().unwrap_or(false);
                    eprintln!("[fluxer-rs] Invalid session (resumable={resumable}).");
                    return Ok(LoopControl::Reconnect { resume: resumable });
                }

                1 => {
                    let seq = *seq_shared.lock().await;
                    let hb = serde_json::json!({ "op": 1, "d": seq });
                    let _ = write
                        .lock()
                        .await
                        .send(WsMessage::Text(hb.to_string().into()))
                        .await;
                }

                _ => {}
            }
        }

        Err(ClientError::ConnectionClosed)
    }
}

async fn cache_update(cache: &Cache, event_type: &str, data: &Value) {
    match event_type {
        "READY" => {
            if let Ok(user) = serde_json::from_value::<crate::model::User>(data["user"].clone()) {
                *cache.current_user.write().await = Some(user);
            }
        }
        "GUILD_CREATE" => {
            if let Ok(guild) = serde_json::from_value::<crate::model::Guild>(data.clone()) {
                let guild_id = guild.id.clone();

                if let Some(channels) = data["channels"].as_array() {
                    let mut ch_map = cache.channels.write().await;
                    for ch_val in channels {
                        if let Ok(ch) = serde_json::from_value::<crate::model::Channel>(ch_val.clone()) {
                            ch_map.insert(ch.id.clone(), ch);
                        }
                    }
                }
                
                if let Some(members) = data["members"].as_array() {
                    let mut user_map = cache.users.write().await;
                    for m_val in members {
                        if let Ok(user) = serde_json::from_value::<crate::model::User>(m_val["user"].clone()) {
                            user_map.insert(user.id.clone(), user);
                        }
                    }
                }
                cache.guilds.write().await.insert(guild_id, guild);
            }
        }
        "GUILD_SYNC" => {
            if let Some(channels) = data["channels"].as_array() {
                let mut ch_map = cache.channels.write().await;
                for ch_val in channels {
                    if let Ok(ch) = serde_json::from_value::<crate::model::Channel>(ch_val.clone()) {
                        ch_map.insert(ch.id.clone(), ch);
                    }
                }
            }
            if let Some(members) = data["members"].as_array() {
                let mut user_map = cache.users.write().await;
                for m_val in members {
                    if let Ok(user) = serde_json::from_value::<crate::model::User>(m_val["user"].clone()) {
                        user_map.insert(user.id.clone(), user);
                    }
                }
            }
        }
        "GUILD_UPDATE" => {
            if let Ok(guild) = serde_json::from_value::<crate::model::Guild>(data.clone()) {
                cache.guilds.write().await.insert(guild.id.clone(), guild);
            }
        }
        "GUILD_DELETE" => {
            if let Some(id) = data["id"].as_str() {
                cache.guilds.write().await.remove(id);
            }
        }
        "CHANNEL_CREATE" | "CHANNEL_UPDATE" => {
            if let Ok(ch) = serde_json::from_value::<crate::model::Channel>(data.clone()) {
                cache.channels.write().await.insert(ch.id.clone(), ch);
            }
        }
        "CHANNEL_DELETE" => {
            if let Some(id) = data["id"].as_str() {
                cache.channels.write().await.remove(id);
            }
        }
        "GUILD_MEMBER_ADD" | "GUILD_MEMBER_UPDATE" => {
            if let Ok(user) = serde_json::from_value::<crate::model::User>(data["user"].clone()) {
                cache.users.write().await.insert(user.id.clone(), user);
            }
        }
        "MESSAGE_CREATE" => {
            if let Ok(user) = serde_json::from_value::<crate::model::User>(data["author"].clone()) {
                cache.users.write().await.insert(user.id.clone(), user);
            }
        }
        _ => {}
    }
}

async fn dispatch_event(
    event_type: String,
    data: Value,
    ctx: Context,
    handler: Arc<dyn EventHandler>,
) {
    use crate::model::{
        AuthSessionChange, CallCreate, CallDelete, CallUpdate, Channel, ChannelPinsAck,
        ChannelPinsUpdate, ChannelRecipientAdd, ChannelRecipientRemove, ChannelUpdateBulk, Guild,
        GuildAuditLogEntryCreate, GuildBanAdd, GuildBanRemove, GuildEmojisUpdate, GuildMemberAdd,
        GuildMemberListUpdate, GuildMemberRemove, GuildMembersChunk, GuildMemberUpdate,
        GuildRoleCreate, GuildRoleDelete, GuildRoleUpdate, GuildRoleUpdateBulk, GuildStickersUpdate,
        GuildSync, InviteCreate, InviteDelete, Message, MessageAck, MessageDelete, MessageDeleteBulk,
        MessageReactionAddMany, MessageUpdate, PassiveUpdates, PresenceUpdate, PresenceUpdateBulk,
        ReactionAdd, ReactionRemove, ReactionRemoveAll, ReactionRemoveEmoji, Ready,
        RecentMentionDelete, RelationshipAdd, RelationshipRemove, RelationshipUpdate, Resumed,
        SavedMessageCreate, SavedMessageDelete, SessionsReplace, TypingStart, UnavailableGuild,
        UserConnectionsUpdate, UserGuildSettingsUpdate, UserNoteUpdate, UserPinnedDmsUpdate,
        UserSettingsUpdate, UserUpdate, VoiceServerUpdate, VoiceStateUpdate, WebhooksUpdate,
    };
    use crate::model::voice::VoiceState;

    macro_rules! dispatch {
        ($method:ident, $ty:ty) => {{
            match serde_json::from_value::<$ty>(data.clone()) {
                Ok(v) => handler.$method(ctx, v).await,
                Err(e) => eprintln!(
                    "[fluxer-rs] Failed to deserialize {} event: {}",
                    stringify!($ty),
                    e
                ),
            }
        }};
    }

    match event_type.as_str() {
        "READY"   => dispatch!(on_ready, Ready),
        "RESUMED" => dispatch!(on_resumed, Resumed),
        "MESSAGE_CREATE"      => dispatch!(on_message, Message),
        "MESSAGE_UPDATE"      => dispatch!(on_message_update, MessageUpdate),
        "MESSAGE_DELETE"      => dispatch!(on_message_delete, MessageDelete),
        "MESSAGE_DELETE_BULK" => dispatch!(on_message_delete_bulk, MessageDeleteBulk),
        "MESSAGE_REACTION_ADD"          => dispatch!(on_reaction_add, ReactionAdd),
        "MESSAGE_REACTION_REMOVE"       => dispatch!(on_reaction_remove, ReactionRemove),
        "MESSAGE_REACTION_REMOVE_ALL"   => dispatch!(on_reaction_remove_all, ReactionRemoveAll),
        "MESSAGE_REACTION_REMOVE_EMOJI" => dispatch!(on_reaction_remove_emoji, ReactionRemoveEmoji),
        "TYPING_START" => dispatch!(on_typing_start, TypingStart),
        "CHANNEL_CREATE"      => dispatch!(on_channel_create, Channel),
        "CHANNEL_UPDATE"      => dispatch!(on_channel_update, Channel),
        "CHANNEL_DELETE"      => dispatch!(on_channel_delete, Channel),
        "CHANNEL_PINS_UPDATE" => dispatch!(on_channel_pins_update, ChannelPinsUpdate),
        "CHANNEL_PINS_ACK"    => dispatch!(on_channel_pins_ack, ChannelPinsAck),
        "GUILD_CREATE" => dispatch!(on_guild_create, Guild),
        "GUILD_UPDATE" => dispatch!(on_guild_update, Guild),
        "GUILD_DELETE" => dispatch!(on_guild_delete, UnavailableGuild),
        "GUILD_MEMBER_ADD"    => dispatch!(on_guild_member_add, GuildMemberAdd),
        "GUILD_MEMBER_UPDATE" => dispatch!(on_guild_member_update, GuildMemberUpdate),
        "GUILD_MEMBER_REMOVE" => dispatch!(on_guild_member_remove, GuildMemberRemove),
        "GUILD_BAN_ADD"    => dispatch!(on_guild_ban_add, GuildBanAdd),
        "GUILD_BAN_REMOVE" => dispatch!(on_guild_ban_remove, GuildBanRemove),
        "GUILD_ROLE_CREATE"      => dispatch!(on_guild_role_create, GuildRoleCreate),
        "GUILD_ROLE_UPDATE"      => dispatch!(on_guild_role_update, GuildRoleUpdate),
        "GUILD_ROLE_UPDATE_BULK" => dispatch!(on_guild_role_update_bulk, GuildRoleUpdateBulk),
        "GUILD_ROLE_DELETE"      => dispatch!(on_guild_role_delete, GuildRoleDelete),
        "GUILD_EMOJIS_UPDATE"   => dispatch!(on_guild_emojis_update, GuildEmojisUpdate),
        "GUILD_STICKERS_UPDATE" => dispatch!(on_guild_stickers_update, GuildStickersUpdate),
        "GUILD_AUDIT_LOG_ENTRY_CREATE" => dispatch!(on_guild_audit_log_entry_create, GuildAuditLogEntryCreate),
        "CHANNEL_UPDATE_BULK" => dispatch!(on_channel_update_bulk, ChannelUpdateBulk),
        "INVITE_CREATE" => dispatch!(on_invite_create, InviteCreate),
        "INVITE_DELETE" => dispatch!(on_invite_delete, InviteDelete),
        "WEBHOOKS_UPDATE" => dispatch!(on_webhooks_update, WebhooksUpdate),
        "VOICE_STATE_UPDATE" => {
            let guild_id = data["guild_id"].as_str().unwrap_or("").to_string();
            let sess = data["session_id"].as_str().unwrap_or("").to_string();
            if !guild_id.is_empty() && !sess.is_empty() {
                let mut states = ctx.voice_states.lock().await;
                let entry = states.entry(guild_id).or_insert_with(|| VoiceState {
                    token: String::new(),
                    endpoint: String::new(),
                    session_id: None,
                });
                entry.session_id = Some(sess);
            }
            match serde_json::from_value::<VoiceStateUpdate>(data.clone()) {
                Ok(v) => handler.on_voice_state_update(ctx, v).await,
                Err(e) => eprintln!("[fluxer-rs] Failed to deserialize VOICE_STATE_UPDATE: {}", e),
            }
        }
        "VOICE_SERVER_UPDATE" => {
            let token = data["token"].as_str().unwrap_or("").to_string();
            let endpoint = data["endpoint"].as_str().unwrap_or("").to_string();
            let guild_id = data["guild_id"].as_str().unwrap_or("").to_string();
            if !guild_id.is_empty() && !token.is_empty() && !endpoint.is_empty() {
                let mut states = ctx.voice_states.lock().await;
                let entry = states.entry(guild_id).or_insert_with(|| VoiceState {
                    token: String::new(),
                    endpoint: String::new(),
                    session_id: None,
                });
                entry.token = token.clone();
                entry.endpoint = if endpoint.starts_with("wss://")
                    || endpoint.starts_with("https://")
                {
                    endpoint.clone()
                } else {
                    format!("wss://{}", endpoint)
                };
            }
            match serde_json::from_value::<VoiceServerUpdate>(data.clone()) {
                Ok(v) => handler.on_voice_server_update(ctx, v).await,
                Err(e) => eprintln!("[fluxer-rs] Failed to deserialize VOICE_SERVER_UPDATE: {}", e),
            }
        }

        "PRESENCE_UPDATE"       => dispatch!(on_presence_update, PresenceUpdate),
        "PRESENCE_UPDATE_BULK"  => dispatch!(on_presence_update_bulk, PresenceUpdateBulk),
        "USER_SETTINGS_UPDATE"  => dispatch!(on_user_settings_update, UserSettingsUpdate),
        "USER_UPDATE"           => dispatch!(on_user_update, UserUpdate),
        "USER_GUILD_SETTINGS_UPDATE" => dispatch!(on_user_guild_settings_update, UserGuildSettingsUpdate),
        "USER_PINNED_DMS_UPDATE"     => dispatch!(on_user_pinned_dms_update, UserPinnedDmsUpdate),
        "USER_NOTE_UPDATE"           => dispatch!(on_user_note_update, UserNoteUpdate),
        "USER_CONNECTIONS_UPDATE"    => dispatch!(on_user_connections_update, UserConnectionsUpdate),
        "AUTH_SESSION_CHANGE"        => dispatch!(on_auth_session_change, AuthSessionChange),
        "MESSAGE_ACK"           => dispatch!(on_message_ack, MessageAck),
        "SESSIONS_REPLACE" => {
            match serde_json::from_value::<Vec<crate::model::SessionEntry>>(data.clone()) {
                Ok(v) => handler.on_sessions_replace(ctx, SessionsReplace(v)).await,
                Err(e) => eprintln!("[fluxer-rs] Failed to deserialize SESSIONS_REPLACE: {}", e),
            }
        }
        "RELATIONSHIP_ADD"    => dispatch!(on_relationship_add, RelationshipAdd),
        "RELATIONSHIP_UPDATE" => dispatch!(on_relationship_update, RelationshipUpdate),
        "RELATIONSHIP_REMOVE" => dispatch!(on_relationship_remove, RelationshipRemove),
        "CALL_CREATE" => dispatch!(on_call_create, CallCreate),
        "CALL_UPDATE" => dispatch!(on_call_update, CallUpdate),
        "CALL_DELETE" => dispatch!(on_call_delete, CallDelete),
        "CHANNEL_RECIPIENT_ADD"    => dispatch!(on_channel_recipient_add, ChannelRecipientAdd),
        "CHANNEL_RECIPIENT_REMOVE" => dispatch!(on_channel_recipient_remove, ChannelRecipientRemove),
        "MESSAGE_REACTION_ADD_MANY" => dispatch!(on_message_reaction_add_many, MessageReactionAddMany),
        "RECENT_MENTION_DELETE"     => dispatch!(on_recent_mention_delete, RecentMentionDelete),
        "SAVED_MESSAGE_CREATE"      => dispatch!(on_saved_message_create, SavedMessageCreate),
        "SAVED_MESSAGE_DELETE"      => dispatch!(on_saved_message_delete, SavedMessageDelete),
        "PASSIVE_UPDATES"           => dispatch!(on_passive_updates, PassiveUpdates),
        "GUILD_MEMBER_LIST_UPDATE"  => dispatch!(on_guild_member_list_update, GuildMemberListUpdate),
        "GUILD_MEMBERS_CHUNK"       => dispatch!(on_guild_members_chunk, GuildMembersChunk),
        "GUILD_SYNC"                => dispatch!(on_guild_sync, GuildSync),

        "INTERACTION_CREATE"
        | "STAGE_INSTANCE_CREATE"
        | "STAGE_INSTANCE_UPDATE"
        | "STAGE_INSTANCE_DELETE" => {}

        other => {
            handler.on_unknown_event(ctx, other.to_string(), data).await;
        }
    }
}