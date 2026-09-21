//! Bot entry point - DI bootstrap + graceful shutdown (analog of App.java).
mod config;
mod core;
mod db;
mod feature;
mod util;

use anyhow::Context;
use core::lifecycle::Lifecycle;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    util::logger::init();

    let cfg = config::loader::load().context("load config")?;
    // Use Lifecycle (avoid dead_code) - Phase 2 wiring placeholder.
    let _ = <() as Lifecycle>::start(&()).await;
    let _ = <() as Lifecycle>::shutdown(&()).await;
    info!(
        prefix = %cfg.prefix,
        guild = ?cfg.guild_id,
        forum = cfg.ticket_use_forum,
        forum_channel = ?cfg.ticket_forum_channel_id,
        "starting bot-kernel"
    );

    let pool = db::pool::create(&cfg.db_path).await.context("db pool")?;
    db::migration::run(&pool).await.context("migrations")?;

    let framework = feature::build_framework(cfg.clone(), pool.clone()).await?;

    let client = core::bot::build_client(&cfg, pool.clone(), framework).await?;
    core::bot::run_with_shutdown(client).await?;
    db::pool::close(pool).await;
    Ok(())
}
