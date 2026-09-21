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
use crate::core::{dispatcher::RateLimiter, policy};
use poise::Framework;
use sqlx::SqlitePool;
use std::sync::OnceLock;

static RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();
fn limiter() -> &'static RateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::new(policy::RATE_LIMIT_WINDOW_SECS))
}

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
            moderation::command::purge(),
            moderation::command::slowmode(),
            moderation::command::warn(),
            moderation::mute::mute(),
            moderation::mute::unmute(),
            blog::command::blog_post(),
            poll::command::poll(),
            remind::command::remind(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(cfg.prefix.clone().into()),
            ..Default::default()
        },
        command_check: Some(|ctx| {
            Box::pin(async move {
                let key = format!("{}:{}", ctx.author().id, ctx.command().qualified_name);
                if let Some(rem) = limiter().check(&key).await {
                    let _ = ctx
                        .send(
                            poise::CreateReply::default()
                                .content(format!("Rate limited — wait {:.1}s", rem.as_secs_f32()))
                                .ephemeral(true),
                        )
                        .await;
                    return Ok(false);
                }
                let (q, _rx) = crate::core::queue::JobQueue::new(
                    policy::QUEUE_CAPACITY,
                    policy::MAX_HANDLER_CONCURRENCY,
                );
                if q.try_submit(|| {}).is_err() {
                    let _ = ctx
                        .send(
                            poise::CreateReply::default()
                                .content("Server busy — try again in 2s")
                                .ephemeral(true),
                        )
                        .await;
                    return Ok(false);
                }
                Ok(true)
            })
        }),
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
