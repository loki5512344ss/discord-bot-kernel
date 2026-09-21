use sqlx::SqlitePool;
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
