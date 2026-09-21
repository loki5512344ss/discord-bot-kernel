//! Forum mode for tickets - one Forum channel with tags instead of category + text channels.
//! Each topic maps to a ForumTag. Controlled by `TICKET_USE_FORUM` env (default false).

use std::collections::HashMap;
use std::sync::OnceLock;

use serenity::all::{
    ChannelId, ChannelType, CreateChannel, CreateForumTag, EditChannel, ForumTagId, GuildId,
};
use serenity::http::Http;
use tracing::{info, warn};

/// Topic → tag display (mirrors `handler::TOPICS` but tag names are forum-specific).
const FORUM_TAGS: &[(&str, &str, char)] = &[
    ("general", "General", '🛠'),
    ("bug", "Bug", '🐛'),
    ("appeal", "Appeal", '🔇'),
    ("report", "Report", '😡'),
    ("owners", "Owners", '👑'),
];

static TAG_CACHE: OnceLock<HashMap<String, ForumTagId>> = OnceLock::new();

/// Returns true if forum mode enabled (`TICKET_USE_FORUM=1/true`).
pub fn is_enabled() -> bool {
    matches!(
        std::env::var("TICKET_USE_FORUM").ok().as_deref().map(|v| v.trim().to_ascii_lowercase()),
        Some(ref s) if s == "1" || s == "true" || s == "yes" || s == "on"
    )
}

/// Ensures forum channel exists (creates if missing) and tags are correct. Returns forum ChannelId.
pub async fn ensure_forum_channel(http: &Http, guild_id: GuildId) -> anyhow::Result<ChannelId> {
    // 1. Try explicit ID from env
    if let Ok(id_str) = std::env::var("TICKET_FORUM_CHANNEL_ID") {
        if let Ok(id) = id_str.parse::<u64>() {
            let cid = ChannelId::new(id);
            if let Ok(ch) = guild_id.channels(http).await {
                if let Some((_, gc)) = ch.iter().find(|(k, _)| **k == cid) {
                    if gc.kind == ChannelType::Forum {
                        sync_tags(http, cid).await?;
                        return Ok(cid);
                    }
                }
            }
        }
    }
    // 2. Search existing forum channels
    let channels = guild_id.channels(http).await?;
    if let Some((id, _)) = channels.iter().find(|(_, c)| {
        c.kind == ChannelType::Forum
            && (c.name.contains("support-forum")
                || c.name.contains("ticket-forum")
                || c.name.contains("tickets"))
    }) {
        sync_tags(http, *id).await?;
        return Ok(*id);
    }
    // 3. Create new forum channel at top (tags synced after via EditChannel)
    let created = guild_id
        .create_channel(
            http,
            CreateChannel::new("🎫-support-forum")
                .kind(ChannelType::Forum)
                .topic("Tickets via forum tags — each post is a ticket")
                .position(0),
        )
        .await?;
    info!(channel=%created.id, "forum channel created for tickets");
    let _ = sync_tags(http, created.id).await;
    Ok(created.id)
}

async fn sync_tags(http: &Http, forum_id: ChannelId) -> anyhow::Result<()> {
    if let Ok(serenity::all::Channel::Guild(gc)) = http.get_channel(forum_id).await {
        let existing: Vec<String> = gc.available_tags.iter().map(|t| t.name.clone()).collect();
        let expected: Vec<String> = FORUM_TAGS.iter().map(|(_, n, _)| n.to_string()).collect();
        let need_sync =
            existing.len() != expected.len() || !expected.iter().all(|e| existing.contains(e));
        if need_sync {
            let tags: Vec<CreateForumTag> = FORUM_TAGS
                .iter()
                .map(|(_, name, emoji)| CreateForumTag::new(*name).emoji(*emoji))
                .collect();
            let _ = forum_id.edit(http, EditChannel::new().available_tags(tags)).await;
            info!(channel=%forum_id, "forum tags synced");
        }
        let mut map = HashMap::new();
        for tag in gc.available_tags {
            map.insert(tag.name.clone(), tag.id);
        }
        let _ = TAG_CACHE.set(map);
    }
    refresh_tag_cache(http, forum_id).await?;
    Ok(())
}

async fn refresh_tag_cache(http: &Http, forum_id: ChannelId) -> anyhow::Result<()> {
    if TAG_CACHE.get().is_some() {
        return Ok(());
    }
    let ch = http.get_channel(forum_id).await?;
    if let serenity::all::Channel::Guild(gc) = ch {
        let mut map = HashMap::new();
        for tag in gc.available_tags {
            map.insert(tag.name.clone(), tag.id);
        }
        let _ = TAG_CACHE.set(map);
    }
    Ok(())
}

/// Returns ForumTagId for topic (e.g. "bug" → tag "Bug"). Requires ensure_forum_channel called before.
pub fn tag_id_for(topic: &str) -> Option<ForumTagId> {
    let want = match topic {
        "general" => "General",
        "bug" => "Bug",
        "appeal" => "Appeal",
        "report" => "Report",
        "owners" => "Owners",
        _ => return None,
    };
    TAG_CACHE.get()?.get(want).copied()
}

/// Also try direct lookup via HTTP if cache miss (best effort sync).
pub async fn tag_id_for_async(http: &Http, forum_id: ChannelId, topic: &str) -> Option<ForumTagId> {
    if let Some(id) = tag_id_for(topic) {
        return Some(id);
    }
    if let Ok(serenity::all::Channel::Guild(gc)) = http.get_channel(forum_id).await {
        let want = match topic {
            "general" => "General",
            "bug" => "Bug",
            "appeal" => "Appeal",
            "report" => "Report",
            "owners" => "Owners",
            _ => return None,
        };
        for tag in gc.available_tags {
            if tag.name == want {
                return Some(tag.id);
            }
        }
    }
    warn!(topic=%topic, "forum tag not found");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tag_mapping() {
        assert_eq!(FORUM_TAGS.len(), 5);
        assert!(FORUM_TAGS.iter().any(|(v, _, _)| *v == "bug"));
    }
}
