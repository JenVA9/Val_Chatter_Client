use serde::Serialize;
use crate::api::client::ApiClient;
use crate::models::Thread;

#[derive(Serialize)]
struct ResolveRequest {
    #[serde(rename = "nodeIds")]
    node_ids: Vec<i64>,
}

pub async fn resolve(client: &ApiClient, token: &str, node_ids: Vec<i64>) -> anyhow::Result<Thread> {
    Ok(client.http
        .post(client.url("/api/threads/resolve"))
        .bearer_auth(token)
        .json(&ResolveRequest { node_ids })
        .send().await?
        .error_for_status()?
        .json::<Thread>().await?)
}
