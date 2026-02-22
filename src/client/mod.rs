use std::collections::HashMap;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use crate::error::ClientError;
use crate::event::EventHandler;
use crate::http::Http;
use crate::model::voice::VoiceState;
use std::time::Duration;

const DEFAULT_API_URL: &str = "https://api.fluxer.app/v1";
const DEFAULT_GATEWAY_URL: &str = "wss://gateway.fluxer.app/?v=1&encoding=json";

#[allow(dead_code)]
enum LoopControl {
    Done,
    Reconnect { resume: bool },
}

#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
    pub gateway_tx: Arc<tokio::sync::mpsc::Sender<String>>,
    pub voice_states: Arc<Mutex<HashMap<String, VoiceState>>>,
    pub(crate) live_rooms: Arc<Mutex<HashMap<String, std::sync::Arc<livekit::Room>>>>,
}

impl Context {
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
        )
        .await
        .map_err(|e| ClientError::Voice(e.to_string()))?;

        self.live_rooms.lock().await.insert(guild_id.to_string(), conn.room.clone());

        Ok(conn)
    }

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
}

pub struct ClientBuilder {
    token: String,
    api_url: String,
    handler: Option<Arc<dyn EventHandler>>,
}

impl ClientBuilder {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_url: DEFAULT_API_URL.to_string(),
            handler: None,
        }
    }

    pub fn event_handler(mut self, handler: impl EventHandler + 'static) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }

    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    pub fn build(self) -> Client {
        let http = Arc::new(Http::new(&self.token, self.api_url));
        Client {
            http,
            handler: self.handler.expect("call .event_handler() before .build()"),
        }
    }
}

pub struct Client {
    pub(crate) http: Arc<Http>,
    handler: Arc<dyn EventHandler>,
}

impl Client {
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(token)
    }

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
            gateway_tx: Arc::new(gateway_tx),
            voice_states: Arc::new(Mutex::new(HashMap::new())),
            live_rooms: Arc::new(Mutex::new(HashMap::new())),
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
                    let code = frame.as_ref().and_then(|f| {
                        let c = f.code;
                        Some(u16::from(c))
                    }).unwrap_or(0);
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
                    }

                    tokio::spawn(async move {
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

async fn dispatch_event(
    event_type: String,
    data: Value,
    ctx: Context,
    handler: Arc<dyn EventHandler>,
) {
    use crate::model::{
        Channel, ChannelPinsUpdate, ChannelUpdateBulk, Guild, GuildBanAdd, GuildBanRemove,
        GuildEmojisUpdate, GuildMemberAdd, GuildMemberRemove, GuildMemberUpdate,
        GuildRoleCreate, GuildRoleDelete, GuildRoleUpdate, GuildRoleUpdateBulk,
        GuildStickersUpdate, InviteCreate, InviteDelete, WebhooksUpdate,
        Message, MessageDelete, MessageDeleteBulk, MessageUpdate, ReactionAdd,
        ReactionRemove, ReactionRemoveAll, ReactionRemoveEmoji, Ready, TypingStart,
        UnavailableGuild,
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
        "RESUMED" => eprintln!("[fluxer-rs] Session resumed successfully."),
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
                entry.token = token;
                entry.endpoint = if endpoint.starts_with("wss://")
                    || endpoint.starts_with("https://")
                {
                    endpoint
                } else {
                    format!("wss://{}", endpoint)
                };
            }
        }

        "INTERACTION_CREATE"
        | "SESSIONS_REPLACE"
        | "STAGE_INSTANCE_CREATE"
        | "STAGE_INSTANCE_UPDATE"
        | "STAGE_INSTANCE_DELETE" => {}

        other => {
            eprintln!("[fluxer-rs] Unknown event: {}", other);
        }
    }
}