use serenity::http::Http;
use sqlx::SqlitePool;
use tracing::{info, warn};
#[rustfmt::skip]
pub async fn list_tickets(p: &SqlitePool) -> Vec<(String, String, String)> {
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS tickets (channel_id TEXT PRIMARY KEY, guild_id TEXT NOT NULL, opener_id TEXT NOT NULL)").execute(p).await;
    sqlx::query_as::<_, (String, String, String)>("SELECT channel_id, guild_id, opener_id FROM tickets").fetch_all(p).await.unwrap_or_default()
}
#[rustfmt::skip]
pub async fn count_tickets(p: &SqlitePool, g: u64) -> i64 {
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS tickets (channel_id TEXT PRIMARY KEY, guild_id TEXT NOT NULL, opener_id TEXT NOT NULL)").execute(p).await;
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tickets WHERE guild_id=?").bind(g.to_string()).fetch_one(p).await.unwrap_or(0)
}
pub async fn reconcile_orphans(http: &Http, pool: &SqlitePool, guild_id: u64) -> usize {
    let tickets = list_tickets(pool).await;
    let mut purged = 0;
    for (cid_str, gid_str, opener) in tickets {
        if gid_str != guild_id.to_string() {
            continue;
        }
        if let Ok(cid) = cid_str.parse::<u64>() {
            let cid = serenity::all::ChannelId::new(cid);
            if http.get_channel(cid).await.is_err() {
                let _ = sqlx::query("DELETE FROM tickets WHERE channel_id=?")
                    .bind(&cid_str)
                    .execute(pool)
                    .await;
                let _ = sqlx::query("DELETE FROM ticket_claims WHERE channel_id=?")
                    .bind(&cid_str)
                    .execute(pool)
                    .await;
                purged += 1;
                warn!(channel=%cid_str, opener=%opener, "orphan ticket purged");
                let _ = serenity::all::ChannelId::new(1509503717127356446).send_message(http, serenity::all::CreateMessage::new().embed(serenity::all::CreateEmbed::new().title("⚠️ Ticket Manually Deleted").description(format!("Ticket <#{cid_str}> (opener <@{opener}>) manually deleted — transcript unavailable.")).color(0x00ED_4245))).await;
            }
        }
    }
    if purged > 0 {
        info!(purged, "orphan tickets reconciled");
    }
    purged
}
