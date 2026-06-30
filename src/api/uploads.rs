use serde::Deserialize;
use reqwest::multipart;
use crate::api::client::ApiClient;

#[derive(Deserialize)]
struct UploadResponse {
    url: String,
}

pub async fn upload_image(
    client: &ApiClient,
    token: &str,
    bytes: Vec<u8>,
    filename: String,
    mime: &str,
) -> anyhow::Result<String> {
    let part = multipart::Part::bytes(bytes)
        .file_name(filename)
        .mime_str(mime)?;
    let form = multipart::Form::new().part("image", part);
    let resp = client.http
        .post(client.url("/api/uploads"))
        .bearer_auth(token)
        .multipart(form)
        .send().await?
        .error_for_status()?
        .json::<UploadResponse>().await?;
    Ok(resp.url)
}
