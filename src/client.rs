use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use futures::{StreamExt, SinkExt};
use serde_json::Value;
use crate::event::EventHandler;
use crate::http::Http;
use crate::model::{Message, Ready, Guild};
use crate::error::ClientError;
use log::{info, warn, error};

const DEFAULT_API_URL: &str = "https://api.fluxer.app/v1";
const HEARTBEAT_INTERVAL_MS: u64 = 30000;
const DEFAULT_INTENTS: u32 = 33280;
const GATEWAY_VERSION: u8 = 1;

#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
}

pub struct Client {
    http: Arc<Http>,
    event_handler: Option<Arc<dyn EventHandler>>,
}

pub struct ClientBuilder {
    token: String,
    api_url: String,
    event_handler: Option<Arc<dyn EventHandler>>,
}

impl ClientBuilder {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_url: DEFAULT_API_URL.to_string(),
            event_handler: None,
        }
    }

    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    pub fn event_handler<H: EventHandler + 'static>(mut self, handler: H) -> Self {
        self.event_handler = Some(Arc::new(handler));
        self
    }

    pub fn build(self) -> Client {
        Client {
            http: Arc::new(Http::new(&self.token, self.api_url)),
            event_handler: self.event_handler,
        }
    }
}

impl Client {
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(token)
    }

    pub async fn start(&mut self) -> Result<(), ClientError> {
        let mut gateway_url = self.http.get_gateway().await?;
        println!("Gateway URL from API: {}", gateway_url);
            
        if !gateway_url.contains("encoding=json") {
            if gateway_url.contains('?') {
                gateway_url.push_str(&format!("&v={}&encoding=json", GATEWAY_VERSION));
            } else {
                gateway_url.push_str(&format!("/?v={}&encoding=json", GATEWAY_VERSION));
            }
        }

        info!("Connecting to gateway: {}", gateway_url);
        let (ws_stream, _) = connect_async(&gateway_url).await?;
        info!("Connected to WebSocket");

        let (write, mut read) = ws_stream.split();
        
        let write = Arc::new(Mutex::new(write));

        let token = self.http.get_token();
        let identify = serde_json::json!({
            "op": 2,
            "d": {
                "token": token,
                "intents": DEFAULT_INTENTS,
                "properties": { "os": "linux", "browser": "fluxer-rs", "device": "fluxer-rs" }
            }
        });

        {
            let mut w = write.lock().await;
            w.send(WsMessage::Text(identify.to_string())).await?;
        }

        let heartbeat_writer = write.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(HEARTBEAT_INTERVAL_MS)).await;
                
                let heartbeat = serde_json::json!({
                    "op": 1,
                    "d": null 
                });

                let mut w = heartbeat_writer.lock().await;
                if let Err(e) = w.send(WsMessage::Text(heartbeat.to_string())).await {
                    println!("Heartbeat failed: {}", e);
                    break;
                }
            }
        });

        if let Some(handler) = &self.event_handler {
            let ctx = Context { http: self.http.clone() };

            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(msg) => {
                        match msg {
                            WsMessage::Text(text) => {
                                let event: Value = match serde_json::from_str(&text) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("Failed to parse event JSON: {}", e);
                                        continue;
                                    }
                                };

                                if let Some(t) = event.get("t").and_then(|v| v.as_str()) {
                                    let data = event["d"].clone();

                                    match t {
                                        "READY" => {
                                            match serde_json::from_value::<Ready>(data) {
                                                Ok(ready) => handler.on_ready(ctx.clone(), ready).await,
                                                Err(e) => warn!("Failed to parse READY: {}", e),
                                            }
                                        }
                                        "MESSAGE_CREATE" => {
                                            if let Ok(message) = serde_json::from_value::<Message>(data) {
                                                handler.on_message(ctx.clone(), message).await;
                                            }
                                        }
                                        "GUILD_CREATE" => {
                                            if let Ok(guild) = serde_json::from_value::<Guild>(data) {
                                                handler.on_guild_create(ctx.clone(), guild).await;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            WsMessage::Close(frame) => {
                                warn!("Server closed connection: {:?}", frame);
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}