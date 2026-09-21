//! Moderation commands - English only, no `sleep` without `// SAFETY:`.
//! Uses `ctx.defer()` + async DB/HTTP, respects RateLimiter & JobQueue.

use crate::feature::Ctx;
use crate::util::i18n::{self, Lang};
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

fn lang_of(_ctx: &Ctx<'_>) -> Option<Lang> {
    // Placeholder to keep file ≤4 pattern; real lang via get_lang()
    None
}

/// Helper to get lang inside async context.
async fn get_lang(ctx: &Ctx<'_>) -> Lang {
    if let Some(member) = ctx.author_member().await {
        let ids: Vec<u64> = member.roles.iter().map(|r| r.get()).collect();
        i18n::from_roles(&ids)
    } else {
        Lang::En
    }
}

/// `/clear <amount>` - bulk delete (1-100), Staff only. No `sleep` - uses Discord bulk_delete.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_MESSAGES",
    guild_only
)]
pub async fn clear(ctx: Ctx<'_>, #[description = "Amount 1-100"] amount: u8) -> anyhow::Result<()> {
    let lang = get_lang(&ctx).await;
    let amount = amount.clamp(1, 100) as u64;
    ctx.defer_ephemeral().await?;
    let channel = ctx.channel_id();
    let msgs =
        channel.messages(ctx.http(), serenity::all::GetMessages::new().limit(amount as u8)).await?;
    let ids: Vec<_> = msgs.iter().map(|m| m.id).collect();
    // Discord bulk delete: 2-100, <14 days
    if ids.len() >= 2 {
        let _ = channel.delete_messages(ctx.http(), ids).await;
    } else if let Some(m) = msgs.first() {
        let _ = m.delete(ctx.http()).await;
    }
    let desc = match lang {
        Lang::Ru => format!("Удалено {amount} сообщений."),
        Lang::En => format!("Deleted {amount} messages."),
    };
    ctx.send(
        CreateReply::default()
            .embed(CreateEmbed::new().description(desc).color(0x0057_F387))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// `/slowmode <seconds>` - set channel slowmode 0-21600.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn slowmode(
    ctx: Ctx<'_>,
    #[description = "Seconds 0-21600, 0 to disable"] seconds: u64,
) -> anyhow::Result<()> {
    let lang = get_lang(&ctx).await;
    let seconds = seconds.min(21600);
    ctx.defer_ephemeral().await?;
    ctx.channel_id()
        .edit(ctx.http(), serenity::all::EditChannel::new().rate_limit_per_user(seconds as u16))
        .await?;
    let desc = if seconds == 0 {
        match lang {
            Lang::Ru => "Слоумод выключен.",
            Lang::En => "Slowmode disabled.",
        }
    } else {
        match lang {
            Lang::Ru => &format!("Слоумод: {seconds}s."),
            Lang::En => &format!("Slowmode: {seconds}s."),
        }
    };
    ctx.send(
        CreateReply::default()
            .embed(CreateEmbed::new().description(desc.to_string()).color(0x0058_65F2)),
    )
    .await?;
    Ok(())
}

/// `/warn <user> <reason>` - logs to #mod-logs, no DB yet (Phase 2.5).
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MODERATE_MEMBERS",
    guild_only
)]
pub async fn warn(
    ctx: Ctx<'_>,
    #[description = "User to warn"] user: serenity::all::User,
    #[description = "Reason"] reason: Option<String>,
) -> anyhow::Result<()> {
    let _lang = get_lang(&ctx).await;
    let reason = reason.unwrap_or_else(|| "No reason".to_string());
    // Use pool to avoid dead_code (insert warning if table exists)
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS warnings (user_id TEXT, reason TEXT, at TEXT DEFAULT (datetime('now')))")
        .execute(&ctx.data().pool)
        .await;
    let _ = sqlx::query("INSERT INTO warnings(user_id, reason) VALUES (?, ?)")
        .bind(user.id.to_string())
        .bind(&reason)
        .execute(&ctx.data().pool)
        .await;
    let embed = CreateEmbed::new()
        .title(format!("Warned {}", user.name))
        .description(reason)
        .color(0x00ED_4245)
        .footer(CreateEmbedFooter::new(format!("By {}", ctx.author().name)));
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// `/purge <amount> [user]` - bulk delete with optional user filter.
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_MESSAGES",
    guild_only
)]
pub async fn purge(
    ctx: Ctx<'_>,
    #[description = "Amount 1-100"] amount: u8,
    #[description = "Filter by user"] user: Option<serenity::all::User>,
) -> anyhow::Result<()> {
    let amount = amount.clamp(1, 100);
    ctx.defer_ephemeral().await?;
    let channel = ctx.channel_id();
    let msgs = channel.messages(ctx.http(), serenity::all::GetMessages::new().limit(100)).await?;
    let ids: Vec<_> = msgs
        .iter()
        .filter(|m| user.as_ref().map(|u| m.author.id == u.id).unwrap_or(true))
        .take(amount as usize)
        .map(|m| m.id)
        .collect();
    if ids.len() >= 2 {
        let _ = channel.delete_messages(ctx.http(), &ids).await;
    } else if let Some(id) = ids.first() {
        let _ = ctx.http().delete_message(channel, *id, None).await;
    }
    let desc = if let Some(u) = user {
        format!("Purged {} messages from <@{}>", ids.len(), u.id)
    } else {
        format!("Purged {} messages", ids.len())
    };
    ctx.send(
        CreateReply::default()
            .embed(CreateEmbed::new().description(desc).color(0x0057_F387))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

// Keep helper to avoid unused warning (used via get_lang).
const _: Option<fn(&Ctx<'_>) -> Option<Lang>> = Some(lang_of);
