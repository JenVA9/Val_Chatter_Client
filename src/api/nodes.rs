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
