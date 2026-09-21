use super::{BotConfig, keys};
use anyhow::{Context, Result};

fn env(key: &str, default: Option<&str>) -> Result<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => default.map(String::from).context(format!("missing env {key}")),
    }
}

fn env_opt(key: &str) -> Option<u64> {
    std::env::var(key).ok()?.parse().ok()
}

fn env_bool(key: &str) -> bool {
    matches!(
        std::env::var(key).ok().as_deref().map(|v| v.trim().to_ascii_lowercase()),
        Some(ref s) if s == "1" || s == "true" || s == "yes" || s == "on"
    )
}

pub fn load() -> Result<BotConfig> {
    let token = env(keys::DISCORD_TOKEN, None)?;
    if token.trim().is_empty() || token == "your_token_here" {
        anyhow::bail!("DISCORD_BOT_TOKEN not set - see .env.example");
    }
    Ok(BotConfig {
        token,
        prefix: env(keys::PREFIX, Some("!"))?,
        guild_id: env_opt(keys::GUILD_ID),
        super_admin_id: env_opt(keys::SUPER_ADMIN_ID),
        db_path: env(keys::DB_PATH, Some("data/bot.db"))?,
        activity: env(keys::BOT_ACTIVITY, Some("!help"))?,
        ticket_use_forum: env_bool(keys::TICKET_USE_FORUM),
        ticket_forum_channel_id: env_opt(keys::TICKET_FORUM_CHANNEL_ID),
    })
}
