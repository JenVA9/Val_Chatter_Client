use serde::{Deserialize, Serialize};
use crate::api::client::ApiClient;
use crate::models::Node;

#[derive(Debug, Clone, Deserialize)]
pub struct AdminUser {
    pub id: i64,
    pub username: String,
    #[serde(deserialize_with = "de_bool")]
    pub is_admin: bool,
    #[serde(deserialize_with = "de_bool")]
    pub is_banned: bool,
    #[serde(deserialize_with = "de_bool")]
    pub is_guest: bool,
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

#[derive(Debug, Clone, Default)]
pub struct StorageInfo {
    pub used_bytes: u64,
    pub limit_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct StorageResponse {
    used_bytes: u64,
    limit_bytes: u64,
}

#[derive(Deserialize)]
struct ConfigRow {
    pub key: String,
    pub value: String,
}

pub async fn get_users(client: &ApiClient, token: &str) -> anyhow::Result<Vec<AdminUser>> {
    Ok(client.http
        .get(client.url("/api/admin/users"))
        .bearer_auth(token)
        .send().await?.error_for_status()?
        .json::<Vec<AdminUser>>().await?)
}

pub async fn set_user_banned(client: &ApiClient, token: &str, user_id: i64, banned: bool) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body { is_banned: bool }
    client.http.put(client.url(&format!("/api/admin/users/{}", user_id)))
        .bearer_auth(token).json(&Body { is_banned: banned })
        .send().await?.error_for_status()?;
    Ok(())
}

pub async fn set_user_admin(client: &ApiClient, token: &str, user_id: i64, admin: bool) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body { is_admin: bool }
    client.http.put(client.url(&format!("/api/admin/users/{}", user_id)))
        .bearer_auth(token).json(&Body { is_admin: admin })
        .send().await?.error_for_status()?;
    Ok(())
}

pub async fn delete_user(client: &ApiClient, token: &str, user_id: i64) -> anyhow::Result<()> {
    client.http.delete(client.url(&format!("/api/admin/users/{}", user_id)))
        .bearer_auth(token).send().await?.error_for_status()?;
    Ok(())
}

pub async fn get_config(client: &ApiClient, token: &str) -> anyhow::Result<Vec<(String, String)>> {
    let rows: Vec<ConfigRow> = client.http
        .get(client.url("/api/admin/config"))
        .bearer_auth(token)
        .send().await?.error_for_status()?
        .json::<Vec<ConfigRow>>().await?;
    Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
}

pub async fn set_config(client: &ApiClient, token: &str, key: &str, value: &str) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body<'a> { value: &'a str }
    client.http.put(client.url(&format!("/api/admin/config/{}", key)))
        .bearer_auth(token).json(&Body { value })
        .send().await?.error_for_status()?;
    Ok(())
}

pub async fn get_nodes(client: &ApiClient, token: &str) -> anyhow::Result<Vec<Node>> {
    Ok(client.http.get(client.url("/api/admin/nodes"))
        .bearer_auth(token).send().await?.error_for_status()?
        .json::<Vec<Node>>().await?)
}

pub async fn create_node(client: &ApiClient, token: &str, node_type: &str, name: &str, parent_id: Option<i64>) -> anyhow::Result<Node> {
    #[derive(Serialize)] struct Body<'a> { #[serde(rename="type")] t: &'a str, name: &'a str, parent_id: Option<i64> }
    Ok(client.http.post(client.url("/api/admin/nodes"))
        .bearer_auth(token).json(&Body { t: node_type, name, parent_id })
        .send().await?.error_for_status()?
        .json::<Node>().await?)
}

pub async fn rename_node(client: &ApiClient, token: &str, node_id: i64, name: &str) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body<'a> { name: &'a str }
    client.http.put(client.url(&format!("/api/admin/nodes/{}", node_id)))
        .bearer_auth(token).json(&Body { name })
        .send().await?.error_for_status()?;
    Ok(())
}

pub async fn delete_node(client: &ApiClient, token: &str, node_id: i64) -> anyhow::Result<()> {
    client.http.delete(client.url(&format!("/api/admin/nodes/{}", node_id)))
        .bearer_auth(token).send().await?.error_for_status()?;
    Ok(())
}

pub async fn get_storage(client: &ApiClient, token: &str) -> anyhow::Result<StorageInfo> {
    let r: StorageResponse = client.http.get(client.url("/api/admin/storage"))
        .bearer_auth(token).send().await?.error_for_status()?
        .json().await?;
    Ok(StorageInfo { used_bytes: r.used_bytes, limit_bytes: r.limit_bytes })
}

pub async fn purge_storage(client: &ApiClient, token: &str) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body { category: &'static str }
    client.http.post(client.url("/api/admin/storage/purge"))
        .bearer_auth(token).json(&Body { category: "all" })
        .send().await?.error_for_status()?;
    Ok(())
}

pub async fn get_ip_bans(client: &ApiClient, token: &str) -> anyhow::Result<Vec<String>> {
    #[derive(Deserialize)] struct IpBan { ip: String }
    let rows: Vec<IpBan> = client.http.get(client.url("/api/admin/ip-bans"))
        .bearer_auth(token).send().await?.error_for_status()?
        .json().await?;
    Ok(rows.into_iter().map(|r| r.ip).collect())
}

pub async fn add_ip_ban(client: &ApiClient, token: &str, ip: &str) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body<'a> { ip: &'a str }
    client.http.post(client.url("/api/admin/ip-bans"))
        .bearer_auth(token).json(&Body { ip })
        .send().await?.error_for_status()?;
    Ok(())
}

pub async fn remove_ip_ban(client: &ApiClient, token: &str, ip: &str) -> anyhow::Result<()> {
    client.http.delete(client.url(&format!("/api/admin/ip-bans/{}", ip)))
        .bearer_auth(token).send().await?.error_for_status()?;
    Ok(())
}
