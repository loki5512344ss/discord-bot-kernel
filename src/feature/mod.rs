pub mod admin;
pub mod blog;
pub mod counter;
pub mod help;
pub mod info;
pub mod lang;
pub mod moderation;
pub mod poll;
pub mod remind;
pub mod ticket;
pub mod verify;
pub mod voice;
pub mod welcome;

use crate::config::BotConfig;
use poise::Framework;
use sqlx::SqlitePool;

pub struct Data {
    pub cfg: BotConfig,
    pub pool: SqlitePool,
}

pub type Ctx<'a> = poise::Context<'a, Data, anyhow::Error>;

pub async fn build_framework(
    cfg: BotConfig,
    pool: SqlitePool,
) -> anyhow::Result<Framework<Data, anyhow::Error>> {
    // Share pool with component handlers (ticket transcript/blacklist/claim).
    crate::feature::ticket::service::set_pool(pool.clone());
    let opts = poise::FrameworkOptions {
        commands: vec![
            help::command::help(),
            info::command::ping(),
            info::command::serverinfo(),
            admin::command::shutdown(),
            verify::command::verify_setup(),
            ticket::command::ticket_panel(),
            ticket::command::ticket_blacklist_add(),
            ticket::command::ticket_blacklist_remove(),
            ticket::command::ticket_add(),
            ticket::command::ticket_remove(),
            moderation::command::clear(),
            moderation::command::slowmode(),
            moderation::command::warn(),
            blog::command::blog_post(),
            poll::command::poll(),
            remind::command::remind(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(cfg.prefix.clone().into()),
            ..Default::default()
        },
        ..Default::default()
    };

    let framework = Framework::builder()
        .options(opts)
        .setup(|ctx, _ready, fw| {
            Box::pin(async move {
                if let Some(gid) = cfg.guild_id {
                    let guild = serenity::all::GuildId::new(gid);
                    poise::builtins::register_in_guild(ctx, &fw.options().commands, guild).await?;
                } else {
                    poise::builtins::register_globally(ctx, &fw.options().commands).await?;
                }
                Ok(Data { cfg, pool })
            })
        })
        .build();
    Ok(framework)
}
