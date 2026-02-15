use std::sync::Arc;
use tokio_tungstenite::connect_async;
use futures::{StreamExt, SinkExt};
use serde_json::Value;
use crate::event::EventHandler;
use crate::http::Http;
use crate::model::Message;

const DEFAULT_API_URL: &str = "https://api.fluxer.app/v1";

#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
}

pub struct Client {
    token: String,
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
            token: self.token,
            event_handler: self.event_handler,
        }
    }
}

impl Client {
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(token)
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let gateway_url = self.http.get_gateway().await?;
        let gateway_url_with_version = format!("{}/?v=1", gateway_url);
        
        let (ws_stream, _) = connect_async(&gateway_url_with_version).await?;
        let (mut write, mut read) = ws_stream.split();

        if let Some(msg) = read.next().await {
            match msg? {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    let hello: Value = serde_json::from_str(&text)?;
                    
                    if hello.get("op").and_then(|v| v.as_u64()) != Some(10) {
                        return Err("Expected HELLO from gateway".into());
                    }
                    
                    let identify = serde_json::json!({
                        "op": 2,
                        "d": {
                            "token": self.token,
                            "intents": 0,
                            "properties": {
                                "os": "linux",
                                "browser": "fluxer-rs",
                                "device": "fluxer-rs"
                            }
                        }
                    });

                    write.send(tokio_tungstenite::tungstenite::Message::Text(identify.to_string())).await?;
                    
                    return self.run_event_loop(write, read).await;
                }
                tokio_tungstenite::tungstenite::Message::Close(frame) => {
                    return Err(format!("Gateway connection closed: {:?}", frame).into());
                }
                _ => {
                    return Err("Unexpected message from gateway".into());
                }
            }
        }
        
        Err("No response from gateway".into())
    }

    async fn run_event_loop(
        &self,
        mut write: futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            tokio_tungstenite::tungstenite::Message
        >,
        mut read: futures::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>
        >
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(handler) = &self.event_handler {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        if let Ok(event) = serde_json::from_str::<Value>(&text) {
                            if let Some(op) = event.get("op").and_then(|v| v.as_u64()) {
                                match op {
                                    0 => {
                                        if let Some(t) = event.get("t").and_then(|v| v.as_str()) {
                                            if t == "MESSAGE_CREATE" {
                                                if let Ok(message) = serde_json::from_value::<Message>(event["d"].clone()) {
                                                    handler.on_message(Context { http: self.http.clone() }, message).await;
                                                }
                                            }
                                        }
                                    }
                                    1 => {
                                        let heartbeat = serde_json::json!({"op": 1, "d": null});
                                        write.send(tokio_tungstenite::tungstenite::Message::Text(heartbeat.to_string())).await?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        return Err(Box::new(e));
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
}