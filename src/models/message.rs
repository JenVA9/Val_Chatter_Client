use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub thread_id: i64,
    pub user_id: i64,
    pub username: String,
    pub content: Option<String>,
    pub image_url: Option<String>,
    #[serde(deserialize_with = "de_bool")]
    pub is_pinned: bool,
    pub pin_expires_at: Option<String>,
    pub created_at: String,
}

fn de_bool<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    use serde::Deserialize;
    let v = serde_json::Value::deserialize(d)?;
    Ok(match &v {
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    })
}
