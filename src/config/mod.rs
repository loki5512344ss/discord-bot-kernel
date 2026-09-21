pub mod keys;
pub mod loader;

#[derive(Debug, Clone)]
pub struct BotConfig {
    pub token: String,
    pub prefix: String,
    pub guild_id: Option<u64>,
    pub super_admin_id: Option<u64>,
    pub db_path: String,
    pub activity: String,
    pub ticket_use_forum: bool,
    pub ticket_forum_channel_id: Option<u64>,
}
