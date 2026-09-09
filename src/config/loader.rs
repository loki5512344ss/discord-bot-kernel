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
    })
}
