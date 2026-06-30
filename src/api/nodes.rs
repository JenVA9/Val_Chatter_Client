use serde::Deserialize;
use crate::api::client::ApiClient;
use crate::models::Node;

pub async fn get_all(client: &ApiClient, token: &str) -> anyhow::Result<Vec<Node>> {
    Ok(client.http
        .get(client.url("/api/nodes"))
        .bearer_auth(token)
        .send().await?
        .error_for_status()?
        .json::<Vec<Node>>().await?)
}

pub async fn get_active(client: &ApiClient, token: &str) -> anyhow::Result<Vec<i64>> {
    #[derive(Deserialize)] struct Resp { node_ids: Vec<i64> }
    let r: Resp = client.http
        .get(client.url("/api/nodes/active"))
        .bearer_auth(token)
        .send().await?
        .error_for_status()?
        .json().await?;
    Ok(r.node_ids)
}
