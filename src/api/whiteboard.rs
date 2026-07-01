use serde::{Deserialize, Serialize};
use crate::api::client::ApiClient;
use crate::models::WbObject;

#[derive(Deserialize)]
struct WbResponse {
    data: Option<Vec<WbObject>>,
}

pub async fn get(client: &ApiClient, token: &str, thread_id: i64) -> anyhow::Result<Option<Vec<WbObject>>> {
    let r: WbResponse = client.http
        .get(client.url(&format!("/api/whiteboard/{}", thread_id)))
        .bearer_auth(token)
        .send().await?.error_for_status()?
        .json().await?;
    Ok(r.data)
}

pub async fn save(client: &ApiClient, token: &str, thread_id: i64, objects: &[WbObject]) -> anyhow::Result<()> {
    #[derive(Serialize)] struct Body<'a> { objects: &'a [WbObject] }
    client.http
        .put(client.url(&format!("/api/whiteboard/{}", thread_id)))
        .bearer_auth(token)
        .json(&Body { objects })
        .send().await?.error_for_status()?;
    Ok(())
}
