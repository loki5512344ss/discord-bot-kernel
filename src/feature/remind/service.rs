//! DB access for reminders - insert on `/remind`, poll due ones from the background task.

use sqlx::SqlitePool;

/// A reminder row ready for delivery.
pub struct DueReminder {
    pub id: i64,
    pub user_id: String,
    pub channel_id: String,
    pub message: String,
}

/// Inserts a new reminder due `in_seconds` from now (computed by SQLite, no clock skew).
pub async fn insert(
    pool: &SqlitePool,
    user_id: &str,
    channel_id: &str,
    message: &str,
    in_seconds: u64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO reminders(user_id, channel_id, message, due_at) \
         VALUES (?, ?, ?, datetime('now', '+' || ? || ' seconds'))",
    )
    .bind(user_id)
    .bind(channel_id)
    .bind(message)
    .bind(in_seconds as i64)
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetches reminders due now that have not been sent yet.
pub async fn fetch_due(pool: &SqlitePool) -> anyhow::Result<Vec<DueReminder>> {
    let rows = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, user_id, channel_id, message FROM reminders \
         WHERE sent = 0 AND due_at <= datetime('now')",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, user_id, channel_id, message)| DueReminder { id, user_id, channel_id, message })
        .collect())
}

/// Marks a reminder as sent so it is not delivered twice.
pub async fn mark_sent(pool: &SqlitePool, id: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE reminders SET sent = 1 WHERE id = ?").bind(id).execute(pool).await?;
    Ok(())
}
