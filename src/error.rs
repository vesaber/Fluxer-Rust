use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Connection closed by server")]
    ConnectionClosed,

    #[error("Voice error: {0}")]
    Voice(String),
}