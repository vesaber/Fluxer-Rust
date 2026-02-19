use std::collections::HashMap;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
use crate::error::ClientError;
use crate::event::EventHandler;
use crate::http::Http;
use crate::model::{Guild, Message, Ready};
use crate::model::voice::VoiceState;

const DEFAULT_API_URL: &str = "https://api.fluxer.app/v1";

#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
    pub gateway_tx: Arc<tokio::sync::mpsc::Sender<String>>,
    pub voice_states: Arc<Mutex<HashMap<String, VoiceState>>>,
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

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        let voice_state = loop {
            {
                let states = self.voice_states.lock().await;
                if let Some(vs) = states.get(guild_id) {
                    break vs.clone();
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(ClientError::Voice(
                    "Timed out waiting for VOICE_SERVER_UPDATE".into(),
                ));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };

        let conn = crate::voice::FluxerVoiceConnection::connect(
            &voice_state.endpoint,
            &voice_state.token,
        )
        .await
        .map_err(|e| ClientError::Voice(e.to_string()))?;

        Ok(conn)
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
        let gateway_url = self.http.get_gateway().await?;
        let ws_url = format!("{}/?v=1&encoding=json", gateway_url);

        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        let (gateway_tx, mut gateway_rx) = tokio::sync::mpsc::channel::<String>(32);
        let write_fwd = write.clone();
        tokio::spawn(async move {
            while let Some(msg) = gateway_rx.recv().await {
                let mut guard = write_fwd.lock().await;
                if guard.send(WsMessage::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        });

        let token = self.http.get_token().to_string();
        let identify = serde_json::json!({
            "op": 2,
            "d": {
                "token": token,
                "intents": 32767,
                "properties": {
                    "os": "linux",
                    "browser": "fluxer",
                    "device": "fluxer"
                }
            }
        });
        write
            .lock()
            .await
            .send(WsMessage::Text(identify.to_string().into()))
            .await?;

        let ctx = Context {
            http: self.http.clone(),
            gateway_tx: Arc::new(gateway_tx),
            voice_states: Arc::new(Mutex::new(HashMap::new())),
        };
        let handler = self.handler.clone();

        while let Some(msg_result) = read.next().await {
            match msg_result? {
                WsMessage::Text(text) => {
                    let payload: Value = serde_json::from_str(text.as_str())?;
                    let op = payload["op"].as_u64().unwrap_or(255);

                    match op {
                        10 => {
                            let interval_ms = payload["d"]["heartbeat_interval"]
                                .as_u64()
                                .unwrap_or(41_250);

                            let write_hb = write.clone();
                            tokio::spawn(async move {
                                let mut ticker = tokio::time::interval(
                                    std::time::Duration::from_millis(interval_ms),
                                );
                                loop {
                                    ticker.tick().await;
                                    let heartbeat = serde_json::json!({"op": 1, "d": null});
                                    let mut guard = write_hb.lock().await;
                                    if guard
                                        .send(WsMessage::Text(heartbeat.to_string().into()))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });
                        }

                        0 => {
                            let event_type = payload["t"].as_str().unwrap_or("").to_string();
                            let data = payload["d"].clone();
                            let ctx = ctx.clone();
                            let handler = handler.clone();

                            tokio::spawn(async move {
                                match event_type.as_str() {
                                    "READY" => {
                                        if let Ok(ready) = serde_json::from_value::<Ready>(data) {
                                            handler.on_ready(ctx, ready).await;
                                        }
                                    }
                                    "MESSAGE_CREATE" => {
                                        if let Ok(msg) = serde_json::from_value::<Message>(data) {
                                            handler.on_message(ctx, msg).await;
                                        }
                                    }
                                    "GUILD_CREATE" => {
                                        if let Ok(guild) = serde_json::from_value::<Guild>(data) {
                                            handler.on_guild_create(ctx, guild).await;
                                        }
                                    }
                                    "VOICE_SERVER_UPDATE" => {
                                        let token = data["token"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string();
                                        let endpoint = data["endpoint"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string();
                                        let guild_id = data["guild_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string();

                                        if !guild_id.is_empty()
                                            && !token.is_empty()
                                            && !endpoint.is_empty()
                                        {
                                            let mut states = ctx.voice_states.lock().await;
                                            states.insert(
                                                guild_id,
                                                VoiceState {
                                                    token,
                                                    endpoint,
                                                },
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            });
                        }

                        _ => {}
                    }
                }

                WsMessage::Close(_) => return Err(ClientError::ConnectionClosed),

                _ => {}
            }
        }

        Ok(())
    }
}