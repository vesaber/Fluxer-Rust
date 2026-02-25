//! Error types used across the library.

use thiserror::Error;

/// The error type returned by pretty much everything in the library.
///
/// You can match on the variant to figure out what went wrong. Most of the time
/// you'll see [`Api`](ClientError::Api) for things like missing permissions, or
/// [`ConnectionClosed`](ClientError::ConnectionClosed) when the gateway drops
/// (which the client handles automatically by reconnecting).
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// Not for bad status codes like 403 or 404 -- those show up as
    /// [`Api`](ClientError::Api). This is for transport-level stuff like
    /// DNS failures, TLS errors, timeouts, etc.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Connection closed by server")]
    ConnectionClosed,

    /// The string contains the status and body, like
    /// `"HTTP 403: {\"message\": \"Missing Permissions\"}"`.
    #[error("API error: {0}")]
    Api(String),

    /// Timeout waiting for `VOICE_SERVER_UPDATE`, LiveKit connection failure, etc.
    #[error("Voice error: {0}")]
    Voice(String),
}