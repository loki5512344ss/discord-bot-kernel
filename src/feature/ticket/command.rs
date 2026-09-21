use crate::feature::Ctx;
use crate::feature::ticket::service;
use poise::CreateReply;
use serenity::all::CreateEmbed;

/// Re-posts modern ticket panel (select + modal).
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn ticket_panel(ctx: Ctx<'_>) -> anyhow::Result<()> {
    let embed = crate::feature::ticket::handler::panel_embed();
    let row = crate::feature::ticket::handler::panel_select_row();
    ctx.send(CreateReply::default().embed(embed).components(vec![row])).await?;
    Ok(())
}

/// Blacklists user from creating tickets.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn ticket_blacklist_add(
    ctx: Ctx<'_>,
    #[description = "User to blacklist"] user: serenity::all::User,
    #[description = "Reason"] reason: String,
) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };
    service::add_blacklist(&ctx.data().pool, guild_id.get(), user.id.get(), &reason).await?;
    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .description(format!("Blacklisted <@{}>: {reason}", user.id))
                    .color(0x00ED_4245),
            )
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Removes user from ticket blacklist.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn ticket_blacklist_remove(
    ctx: Ctx<'_>,
    #[description = "User to unblacklist"] user: serenity::all::User,
) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };
    let removed = service::remove_blacklist(&ctx.data().pool, guild_id.get(), user.id.get()).await;
    let msg = if removed {
        format!("Removed blacklist for <@{}>", user.id)
    } else {
        format!("<@{}> was not blacklisted", user.id)
    };
    ctx.send(
        CreateReply::default()
            .embed(CreateEmbed::new().description(msg).color(0x0057_F387))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
