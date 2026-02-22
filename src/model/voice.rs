use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: Option<String>,
}