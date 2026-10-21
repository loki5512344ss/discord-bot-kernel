use crate::feature::Ctx;
use poise::CreateReply;
use serenity::all::CreateEmbed;

// TODO: debug/test-only - удалить после Phase 1 (как просил)
#[poise::command(prefix_command, owners_only)]
pub async fn shutdown(ctx: Ctx<'_>) -> anyhow::Result<()> {
    let cfg = &ctx.data().cfg;
    let author = ctx.author().id.get();
    let allowed = cfg.super_admin_id.is_some_and(|id| id == author);
    if !allowed {
        ctx.send(
            CreateReply::default().embed(
                CreateEmbed::new().title("⛔ No permission").color(crate::util::brand::DANGER),
            ),
        )
        .await?;
        return Ok(());
    }
    ctx.say("Shutting down…").await?;
    ctx.framework().shard_manager.shutdown_all().await;
    Ok(())
}
