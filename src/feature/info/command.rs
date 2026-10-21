use crate::feature::Ctx;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use std::time::Instant;

// TODO: debug/test-only - удалить после Phase 1 (как просил)
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Ctx<'_>) -> anyhow::Result<()> {
    let start = Instant::now();
    ctx.defer().await?;
    let ms = start.elapsed().as_millis();
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Pong!")
                .description(format!("{ms} ms"))
                .color(crate::util::brand::OK),
        ),
    )
    .await?;
    Ok(())
}

/// Shows server info + touches DB pool to avoid dead_code (uses `Data.pool`).
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn serverinfo(ctx: Ctx<'_>) -> anyhow::Result<()> {
    let guild = ctx.guild().ok_or_else(|| anyhow::anyhow!("guild only"))?.clone();
    let members = guild.member_count;
    // Use DB pool (prove Lifecycle/pool wiring, English message).
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _migrations")
        .fetch_one(&ctx.data().pool)
        .await
        .unwrap_or(0);
    let embed = CreateEmbed::new()
        .title(&guild.name)
        .description(format!("ID: `{}` · Members: {members} · Migrations: {count}", guild.id))
        .color(crate::util::brand::ACCENT);
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
