use serde::Serialize;
use crate::api::client::ApiClient;
use crate::models::Message;

#[derive(Serialize)]
struct PinBody {
    duration_minutes: Option<u32>,
}

pub async fn pin(client: &ApiClient, token: &str, message_id: i64) -> anyhow::Result<()> {
    pin_timed(client, token, message_id, None).await
}

pub async fn pin_timed(
    client: &ApiClient,
    token: &str,
    message_id: i64,
    duration_minutes: Option<u32>,
) -> anyhow::Result<()> {
    client.http
        .post(client.url(&format!("/api/pins/{}", message_id)))
        .bearer_auth(token)
        .json(&PinBody { duration_minutes })
        .send().await?
        .error_for_status()?;
    Ok(())
}

pub async fn unpin(client: &ApiClient, token: &str, message_id: i64) -> anyhow::Result<()> {
    client.http
        .delete(client.url(&format!("/api/pins/{}", message_id)))
        .bearer_auth(token)
        .send().await?
        .error_for_status()?;
    Ok(())
}

pub async fn get_pinned(client: &ApiClient, token: &str, thread_id: i64) -> anyhow::Result<Vec<Message>> {
    Ok(client.http
        .get(client.url(&format!("/api/pins/{}", thread_id)))
        .bearer_auth(token)
        .send().await?
        .error_for_status()?
        .json::<Vec<Message>>().await?)
}
