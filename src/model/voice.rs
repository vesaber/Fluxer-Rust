use serde::{Deserialize, Serialize};

/// Voice connection state, populated internally from gateway events
/// during the voice handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: Option<String>,
}