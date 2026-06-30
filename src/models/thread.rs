use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: i64,
    pub canonical_key: String,
    #[serde(default = "default_chat_mode")]
    pub mode: String,
    pub created_at: String,
}

fn default_chat_mode() -> String { "chat".to_string() }
