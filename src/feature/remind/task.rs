//! Background loop: delivers due reminders as a mention in their origin channel.

use serenity::all::{ChannelId, Context};
use sqlx::SqlitePool;
use tracing::warn;

const POLL_INTERVAL_SECS: u64 = 30;

/// Spawns the periodic reminder-delivery task. Non-blocking, one instance per boot.
pub fn spawn(ctx: Context, pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = run_once(&ctx, &pool).await {
                warn!(error = %e, "reminder delivery failed");
            }
            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    });
}

async fn run_once(ctx: &Context, pool: &SqlitePool) -> anyhow::Result<()> {
    for r in super::service::fetch_due(pool).await? {
        let (Ok(user_id), Ok(channel_id)) = (r.user_id.parse::<u64>(), r.channel_id.parse::<u64>())
        else {
            super::service::mark_sent(pool, r.id).await?;
            continue;
        };
        let content = format!("⏰ <@{user_id}> reminder: {}", r.message);
        let _ = ChannelId::new(channel_id).say(&ctx.http, content).await;
        super::service::mark_sent(pool, r.id).await?;
    }
    Ok(())
}
