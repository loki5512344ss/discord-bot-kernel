//! Background loop: keeps a voice channel named "👥｜Members: N" in sync.
//! Discord limits channel renames to ~2/10min, so we poll every 10 minutes
//! and skip the edit entirely when the count has not changed.

use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, EditChannel, GuildId};
use tracing::{info, warn};

const VOICE_CATEGORY: ChannelId = ChannelId::new(1509503649062326484);
const MARKER: &str = "👥｜Members:";
const INTERVAL_SECS: u64 = 600;

/// Finds the counter channel by name marker, creating it once if missing.
async fn ensure_channel(ctx: &Context, guild_id: GuildId) -> anyhow::Result<ChannelId> {
    let channels = guild_id.channels(&ctx.http).await?;
    if let Some((id, _)) = channels.iter().find(|(_, c)| c.name.starts_with(MARKER)) {
        return Ok(*id);
    }
    let created = guild_id
        .create_channel(
            &ctx.http,
            CreateChannel::new(format!("{MARKER} 0"))
                .kind(ChannelType::Voice)
                .category(VOICE_CATEGORY),
        )
        .await?;
    info!(channel = %created.id, "member counter channel created");
    Ok(created.id)
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
    let count =
        ctx.http.get_guild_with_counts(guild_id).await?.approximate_member_count.unwrap_or(0);
    if *last_count == Some(count) {
        return Ok(());
    }
    let channel_id = ensure_channel(ctx, guild_id).await?;
    channel_id.edit(&ctx.http, EditChannel::new().name(format!("{MARKER} {count}"))).await?;
    *last_count = Some(count);
    info!(count, "member counter updated");
    Ok(())
}
