//! Backend client for blog integration (`POST /api/discord/*`).
//! Handles image upload (multipart) and news creation (JSON) with `X-Bot-Token` auth.

use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

/// Bilingual field (`{ru, en}`) as expected by backend.
#[derive(Debug, Clone, Serialize)]
pub struct Bilingual {
    /// Russian value.
    pub ru: String,
    /// English value.
    pub en: String,
}

/// Payload for `POST /api/discord/news`.
#[derive(Debug, Clone, Serialize)]
pub struct NewsPayload {
    /// Title bilingual.
    pub title: Bilingual,
    /// Markdown content bilingual.
    pub content: Bilingual,
    /// Optional excerpt bilingual.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<Bilingual>,
    /// Cover image URL from `/discord/upload`.
    #[serde(rename = "coverImage")]
    pub cover_image: Option<String>,
    /// Additional image URLs.
    pub images: Vec<String>,
    /// Tags for filtering.
    pub tags: Vec<String>,
    /// Origin Discord channel id.
    #[serde(rename = "channelId")]
    pub channel_id: String,
    /// Origin Discord message id (idempotency).
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// Author Discord user id.
    #[serde(rename = "authorId")]
    pub author_id: String,
}

/// Response from `POST /api/discord/news`.
#[derive(Debug, Clone, Deserialize)]
pub struct NewsResponse {
    /// Created post id.
    pub id: String,
    /// Generated slug.
    pub slug: String,
    /// Whether new post was created.
    #[serde(default)]
    pub imported: Option<bool>,
    /// Optional message (e.g. "Already imported").
    pub message: Option<String>,
}

/// Response from `POST /api/discord/upload`.
#[derive(Debug, Clone, Deserialize)]
struct UploadResponse {
    url: String,
}

/// Returns backend base URL from `BIO_API_URL` or default.
fn api_base() -> String {
    std::env::var("BIO_API_URL").unwrap_or_else(|_| "http://localhost:3001/api".to_string())
}

/// Returns bot secret from `DISCORD_BOT_SECRET` env.
fn bot_secret() -> anyhow::Result<String> {
    std::env::var("DISCORD_BOT_SECRET").map_err(|_| anyhow::anyhow!("DISCORD_BOT_SECRET not set"))
}

/// Uploads image bytes to `POST /api/discord/upload` (multipart).
///
/// # Arguments
/// * `data` - Raw image bytes
/// * `filename` - Original filename with extension
/// * `mime` - MIME type (e.g. `image/png`)
pub async fn upload_image(data: Vec<u8>, filename: String, mime: String) -> anyhow::Result<String> {
    let secret = bot_secret()?;
    let url = format!("{}/discord/upload", api_base().trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mime_owned = mime.clone();
    let part = Part::bytes(data).file_name(filename.clone()).mime_str(&mime_owned)?;
    let form = Form::new().part("file", part);
    let resp = client.post(&url).header("X-Bot-Token", &secret).multipart(form).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    if status.as_u16() == 401 {
        warn!("upload 401 unauthorized");
        anyhow::bail!("upload unauthorized (401) - check DISCORD_BOT_SECRET");
    }
    if status.as_u16() == 400 {
        error!(body = %body, "upload 400");
        anyhow::bail!("upload rejected (400): {body}");
    }
    if !status.is_success() {
        anyhow::bail!("upload failed {status}: {body}");
    }
    let parsed: UploadResponse = serde_json::from_str(&body)?;
    Ok(parsed.url)
}

/// Posts news to `POST /api/discord/news` with `X-Bot-Token`.
///
/// Handles `401` (bad secret) and `400` (validation) explicitly.
pub async fn post_news(payload: NewsPayload) -> anyhow::Result<NewsResponse> {
    let secret = bot_secret()?;
    let url = format!("{}/discord/news", api_base().trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("X-Bot-Token", &secret)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if status.as_u16() == 401 {
        warn!("post_news 401 unauthorized");
        anyhow::bail!("post_news unauthorized (401) - check DISCORD_BOT_SECRET");
    }
    if status.as_u16() == 400 {
        error!(body = %body, "post_news 400");
        anyhow::bail!("post_news validation failed (400): {body}");
    }
    if !status.is_success() {
        anyhow::bail!("post_news failed {status}: {body}");
    }
    let parsed: NewsResponse = serde_json::from_str(&body)?;
    Ok(parsed)
}
