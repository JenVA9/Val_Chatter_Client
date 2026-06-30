use serde::Serialize;
use crate::api::client::ApiClient;
use crate::models::Message;

pub async fn get(client: &ApiClient, token: &str, thread_id: i64, before: Option<i64>) -> anyhow::Result<Vec<Message>> {
    let mut url = client.url(&format!("/api/messages/{}", thread_id));
    if let Some(b) = before {
        url.push_str(&format!("?before={}", b));
    }
    Ok(client.http
        .get(&url)
        .bearer_auth(token)
        .send().await?
        .error_for_status()?
        .json::<Vec<Message>>().await?)
}

#[derive(Serialize)]
struct SendRequest {
    #[serde(rename = "threadId")]
    thread_id: i64,
    content: Option<String>,
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
}

pub async fn send(
    client: &ApiClient,
    token: &str,
    thread_id: i64,
    content: Option<String>,
    image_url: Option<String>,
) -> anyhow::Result<Message> {
    Ok(client.http
        .post(client.url("/api/messages"))
        .bearer_auth(token)
        .json(&SendRequest { thread_id, content, image_url })
        .send().await?
        .error_for_status()?
        .json::<Message>().await?)
}

pub async fn delete(client: &ApiClient, token: &str, message_id: i64) -> anyhow::Result<()> {
    client.http
        .delete(client.url(&format!("/api/messages/{}", message_id)))
        .bearer_auth(token)
        .send().await?
        .error_for_status()?;
    Ok(())
}
