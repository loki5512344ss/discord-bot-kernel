//! Message handler for blog auto-posting.
//! Watches announcements/releases/github-log and forwards staff messages to backend.

use serenity::all::{Context, CreateMessage, Member, Message};
use tracing::{info, warn};

use super::service::{Bilingual, NewsPayload};

/// Allowed channels for auto-post.
const ANNOUNCEMENTS: u64 = 1509503667332714566;
const RELEASES: u64 = 1509503670163869808;
const GITHUB_LOG: u64 = 1509503672197840996;

/// Staff roles that can trigger blog posts.
const STAFF_ROLES: &[u64] = &[
    1547203753373859901, // Moderator
    1547203757711040572, // Staff
    1547203762035236925, // Owner
];

/// Checks if member has staff role or is super admin.
fn is_staff(member: &Member) -> bool {
    if let Ok(super_id) = std::env::var("SUPER_ADMIN_ID") {
        if let Ok(parsed) = super_id.parse::<u64>() {
            if member.user.id.get() == parsed {
                return true;
            }
        }
    }
    member.roles.iter().any(|r| STAFF_ROLES.contains(&r.get()))
}

/// Extracts title as first non-empty line, fallback to truncated content.
fn extract_title(content: &str) -> String {
    let first = content.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("Untitled");
    let mut t = first.to_string();
    if t.chars().count() > 120 {
        t = t.chars().take(120).collect();
    }
    t
}

/// Creates bilingual wrapper (duplicates to both languages).
fn bilingual(s: &str) -> Bilingual {
    Bilingual { ru: s.to_string(), en: s.to_string() }
}

/// Builds excerpt from content (first 200 chars).
fn excerpt_of(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.chars().count() <= 200 {
        trimmed.to_string()
    } else {
        let mut e: String = trimmed.chars().take(200).collect();
        e.push('…');
        e
    }
}

/// Extracts tags from hashtags and channel default.
fn extract_tags(content: &str, channel_id: serenity::all::ChannelId) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    // Channel default tags.
    let default = match channel_id.get() {
        ANNOUNCEMENTS => Some("announcement"),
        RELEASES => Some("release"),
        GITHUB_LOG => Some("github"),
        _ => None,
    };
    if let Some(d) = default {
        tags.push(d.to_string());
    }
    // Hashtags: #word (alphanumeric + - _)
    for word in content.split_whitespace() {
        if let Some(stripped) = word.strip_prefix('#') {
            let clean: String = stripped
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !clean.is_empty() && clean.len() <= 24 {
                let lower = clean.to_lowercase();
                if !tags.contains(&lower) {
                    tags.push(lower);
                }
            }
        }
    }
    tags
}

/// Returns true if channel is monitored for blog posts.
fn is_allowed_channel(id: u64) -> bool {
    matches!(id, ANNOUNCEMENTS | RELEASES | GITHUB_LOG)
}

/// Checks if attachment MIME is an image.
fn is_image(mime: &Option<String>) -> bool {
    if let Some(m) = mime { m.starts_with("image/") } else { false }
}

/// Processes a single message: uploads attachments then posts news.
async fn process_message(ctx: Context, msg: Message) -> anyhow::Result<()> {
    let title = extract_title(&msg.content);
    let content = if msg.content.trim().is_empty() { title.clone() } else { msg.content.clone() };
    let tags = extract_tags(&msg.content, msg.channel_id);
    // Upload attachments.
    let mut images: Vec<String> = Vec::new();
    let mut cover_image: Option<String> = None;
    for att in &msg.attachments {
        if !is_image(&att.content_type) {
            continue;
        }
        // Download attachment bytes via its URL.
        let bytes = download_bytes(&att.url).await?;
        let mime = att.content_type.clone().unwrap_or_else(|| "image/png".to_string());
        match super::service::upload_image(bytes, att.filename.clone(), mime).await {
            Ok(url) => {
                if cover_image.is_none() {
                    cover_image = Some(url.clone());
                }
                images.push(url);
                info!(filename = %att.filename, "image uploaded");
            }
            Err(e) => warn!(error = %e, filename = %att.filename, "image upload failed"),
        }
    }
    let payload = NewsPayload {
        title: bilingual(&title),
        content: bilingual(&content),
        excerpt: Some(bilingual(&excerpt_of(&content))),
        cover_image,
        images,
        tags,
        channel_id: msg.channel_id.to_string(),
        message_id: msg.id.to_string(),
        author_id: msg.author.id.to_string(),
    };
    let resp = super::service::post_news(payload).await?;
    info!(
        id = %resp.id,
        slug = %resp.slug,
        imported = ?resp.imported,
        msg = ?resp.message,
        channel = %msg.channel_id,
        message = %msg.id,
        "blog post created from discord"
    );
    // Acknowledge in channel (best-effort).
    let _ = msg
        .channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new().content(format!("✅ Blog posted: `{}`", resp.slug)),
        )
        .await;
    Ok(())
}

/// Downloads bytes from URL via reqwest.
async fn download_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = reqwest::Client::new().get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("download failed {}", resp.status());
    }
    Ok(resp.bytes().await?.to_vec())
}

/// Entry point for `EventHandler::message`. Non-blocking via `tokio::spawn`.
pub async fn on_message(ctx: Context, msg: Message) {
    if msg.author.bot {
        return;
    }
    if msg.content.trim().is_empty() && msg.attachments.is_empty() {
        return;
    }
    if !is_allowed_channel(msg.channel_id.get()) {
        return;
    }
    let Some(guild_id) = msg.guild_id else {
        return;
    };
    let member = match guild_id.member(&ctx.http, msg.author.id).await {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, user = %msg.author.id, "failed to fetch member");
            return;
        }
    };
    if !is_staff(&member) {
        return;
    }
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        if let Err(e) = process_message(ctx_clone, msg).await {
            warn!(error = %e, "blog process_message failed");
        }
    });
}
