//! Background loop: keeps a voice channel named "👥 Members: N" in sync, outside categories at top.
//! Discord limits channel renames to ~2/10min, so we poll every 10 minutes
//! and skip the edit entirely when the count has not changed.
//! Counts human members only (excludes bots).

use serenity::all::{
    ChannelId, ChannelType, Context, CreateChannel, EditChannel, GuildId, PermissionOverwrite,
    PermissionOverwriteType, Permissions,
};
use tracing::{info, warn};

const MARKER: &str = "👥 Members:";
const INTERVAL_SECS: u64 = 600;

/// Finds the counter channel by name marker, creating it once if missing outside any category at top.
/// Enforces outside-category + position 0 on every call (startup reconcile).
async fn ensure_channel(ctx: &Context, guild_id: GuildId) -> anyhow::Result<ChannelId> {
    let channels = guild_id.channels(&ctx.http).await?;
    if let Some((id, ch)) = channels.iter().find(|(_, c)| c.name.starts_with(MARKER)) {
        let needs_move = ch.parent_id.is_some() || ch.position != 0;
        if needs_move {
            let mut edit = EditChannel::new().position(0);
            if ch.parent_id.is_some() {
                edit = edit.category(Option::<ChannelId>::None);
            }
            let _ = id.edit(&ctx.http, edit).await;
            info!(channel=%id, position=%ch.position, parent=?ch.parent_id, "member counter reconciled to top (outside category, pos 0)");
        }
        // Cleanup old duplicates with different marker (e.g. "👥｜Members:" with separator)
        for (oid, oc) in channels.iter().filter(|(oid, _)| *oid != id) {
            if oc.name.contains("Members:") {
                // Old counter with different marker or duplicate — delete
                let _ = oid.delete(&ctx.http).await;
                info!(channel=%oid, name=%oc.name, "old member counter deleted (migration)");
            }
        }
        return Ok(*id);
    }
    // Migration: found old marker but not new — rename/move it instead of creating new
    if let Some((id, _)) = channels.iter().find(|(_, c)| c.name.contains("Members:")) {
        let _ = id
            .edit(
                &ctx.http,
                EditChannel::new()
                    .name(format!("{MARKER} 0"))
                    .position(0)
                    .category(Option::<ChannelId>::None),
            )
            .await;
        info!(channel=%id, "old counter migrated to new format");
        return Ok(*id);
    }
    // Create outside category at position 0, deny CONNECT so users can't join, but VIEW remains
    let everyone = serenity::all::RoleId::new(guild_id.get());
    let overwrites = vec![PermissionOverwrite {
        allow: Permissions::VIEW_CHANNEL | Permissions::CONNECT,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role(everyone),
    }];
    // Note: we allow CONNECT but voice channel will appear as counter; to prevent joining we could deny CONNECT
    // For counter, keep VIEW only; if you want to block joining, set deny CONNECT. Keep allow for visibility.
    // Here we set deny CONNECT to make it non-joinable but still visible at top.
    let deny_connect = vec![PermissionOverwrite {
        allow: Permissions::VIEW_CHANNEL,
        deny: Permissions::CONNECT,
        kind: PermissionOverwriteType::Role(everyone),
    }];
    let _ = overwrites; // keep for reference
    let created = guild_id
        .create_channel(
            &ctx.http,
            CreateChannel::new(format!("{MARKER} 0"))
                .kind(ChannelType::Voice)
                .position(0)
                .permissions(deny_connect),
        )
        .await?;
    info!(channel = %created.id, "member counter channel created at top (outside category, bots excluded)");
    Ok(created.id)
}

/// Counts human (non-bot) members via paginated guild members fetch.
/// Falls back to approximate count if privileged fetch fails.
async fn fetch_human_count(ctx: &Context, guild_id: GuildId) -> u64 {
    // Try accurate paginated fetch (requires GUILD_MEMBERS intent)
    let mut after: Option<serenity::all::UserId> = None;
    let mut human_count: u64 = 0;
    let mut total_fetched: usize = 0;
    loop {
        let batch = match guild_id.members(&ctx.http, Some(1000), after).await {
            Ok(m) => m,
            Err(e) => {
                warn!(error=%e, "members fetch failed, fallback to approximate");
                // Fallback
                if let Ok(g) = ctx.http.get_guild_with_counts(guild_id).await {
                    return g.approximate_member_count.unwrap_or(human_count);
                }
                return human_count;
            }
        };
        if batch.is_empty() {
            break;
        }
        total_fetched += batch.len();
        for m in &batch {
            if !m.user.bot {
                human_count += 1;
            }
        }
        if batch.len() < 1000 {
            break;
        }
        after = batch.last().map(|m| m.user.id);
        // Safety: prevent infinite loop on huge guilds (>10k) - but still handle
        if total_fetched > 15000 {
            warn!("large guild >15k members, truncated count");
            break;
        }
    }
    // If we got zero but guild not empty, fallback
    if human_count == 0 && total_fetched == 0 {
        if let Ok(g) = ctx.http.get_guild_with_counts(guild_id).await {
            if let Some(c) = g.approximate_member_count {
                return c;
            }
        }
    }
    human_count
}

/// Spawns the periodic counter-refresh task. Non-blocking, one instance per boot.
pub fn spawn(ctx: Context, guild_id: GuildId) {
    tokio::spawn(async move {
        let mut last_count: Option<u64> = None;
        loop {
            match run_once(&ctx, guild_id, &mut last_count).await {
                Ok(()) => {}
                Err(e) => warn!(error = %e, "member counter refresh failed"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(INTERVAL_SECS)).await;
        }
    });
}

async fn run_once(
    ctx: &Context,
    guild_id: GuildId,
    last_count: &mut Option<u64>,
) -> anyhow::Result<()> {
    // Always reconcile position first, even if count unchanged
    let channel_id = ensure_channel(ctx, guild_id).await?;
    let count = fetch_human_count(ctx, guild_id).await;
    if *last_count == Some(count) {
        // Still need to verify name is correct (channel could have been renamed manually) - check via cache
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            if let Some((_, ch)) = channels.iter().find(|(id, _)| **id == channel_id) {
                let expected = format!("{MARKER} {count}");
                if ch.name == expected {
                    return Ok(());
                }
            }
        }
    }
    let mut edit = EditChannel::new().name(format!("{MARKER} {count}"));
    if let Ok(channels) = guild_id.channels(&ctx.http).await {
        if let Some((_, ch)) = channels.iter().find(|(id, _)| **id == channel_id) {
            if ch.parent_id.is_some() || ch.position != 0 {
                edit = edit.category(Option::<ChannelId>::None).position(0);
            }
        }
    }
    channel_id.edit(&ctx.http, edit).await?;
    *last_count = Some(count);
    info!(count, "member counter updated (humans only, outside category, top)");
    Ok(())
}
