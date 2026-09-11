use sqlx::SqlitePool;
use tracing::info;

pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    let migrations =
        [("V1__initial", V1_INITIAL), ("V2__tickets", V2_TICKETS), ("V3__reminders", V3_REMINDERS)];
    for (name, sql) in migrations {
        let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _migrations WHERE name = ?")
            .bind(name)
            .fetch_one(pool)
            .await?;
        if exists == 0 {
            let mut tx = pool.begin().await?;
            for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
                sqlx::query(stmt).execute(&mut *tx).await?;
            }
            sqlx::query("INSERT INTO _migrations(name) VALUES(?)")
                .bind(name)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            info!(migration = name, "applied");
        }
    }
    Ok(())
}

const V1_INITIAL: &str = "
CREATE TABLE IF NOT EXISTS guild_config (
    guild_id TEXT PRIMARY KEY,
    prefix TEXT NOT NULL DEFAULT '!',
    lang TEXT NOT NULL DEFAULT 'ru'
);
CREATE TABLE IF NOT EXISTS user_stats (
    user_id TEXT PRIMARY KEY,
    xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 0,
    message_count INTEGER NOT NULL DEFAULT 0
);
";
const V2_TICKETS: &str = "
CREATE TABLE IF NOT EXISTS ticket_blacklist (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    PRIMARY KEY (guild_id, user_id)
);
CREATE TABLE IF NOT EXISTS ticket_claims (
    channel_id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    claimed_by TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tickets (
    channel_id TEXT PRIMARY KEY,
    guild_id TEXT NOT NULL,
    opener_id TEXT NOT NULL
);
";
const V3_REMINDERS: &str = "
CREATE TABLE IF NOT EXISTS reminders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message TEXT NOT NULL,
    due_at TEXT NOT NULL,
    sent INTEGER NOT NULL DEFAULT 0
);
";
