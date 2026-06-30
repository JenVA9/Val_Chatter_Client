use serde::{Deserialize, Serialize};
use crate::api::client::ApiClient;

#[derive(Serialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub token: String,
    #[serde(rename = "userId")]
    pub user_id: i64,
    pub username: String,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_guest: bool,
}

pub async fn login(client: &ApiClient, username: &str, password: &str) -> anyhow::Result<AuthResponse> {
    Ok(client.http
        .post(client.url("/api/auth/login"))
        .json(&Credentials { username: username.into(), password: password.into() })
        .send().await?
        .error_for_status()?
        .json::<AuthResponse>().await?)
}

pub async fn register(client: &ApiClient, username: &str, password: &str) -> anyhow::Result<AuthResponse> {
    Ok(client.http
        .post(client.url("/api/auth/register"))
        .json(&Credentials { username: username.into(), password: password.into() })
        .send().await?
        .error_for_status()?
        .json::<AuthResponse>().await?)
}
