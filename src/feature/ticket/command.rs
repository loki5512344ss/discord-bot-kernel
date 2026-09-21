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

/// Adds user to current ticket (staff only, must be ticket channel).
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn ticket_add(
    ctx: Ctx<'_>,
    #[description = "User to add"] user: serenity::all::User,
) -> anyhow::Result<()> {
    let channel_id = ctx.channel_id();
    if let Some(pool) = service::pool() {
        if service::get_opener(pool, channel_id.get()).await.is_none() {
            ctx.send(CreateReply::default().content("Not a ticket channel.").ephemeral(true))
                .await?;
            return Ok(());
        }
    }
    service::add_user_to_ticket(ctx.http(), channel_id, user.id.get()).await?;
    let _ = channel_id
        .send_message(
            ctx.http(),
            serenity::all::CreateMessage::new().content(format!(
                "<@{}> added <@{}> to ticket",
                ctx.author().id,
                user.id
            )),
        )
        .await;
    ctx.send(CreateReply::default().content(format!("Added <@{}>", user.id)).ephemeral(true))
        .await?;
    Ok(())
}

/// Removes user from current ticket.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn ticket_remove(
    ctx: Ctx<'_>,
    #[description = "User to remove"] user: serenity::all::User,
) -> anyhow::Result<()> {
    let channel_id = ctx.channel_id();
    if let Some(pool) = service::pool() {
        if service::get_opener(pool, channel_id.get()).await.is_none() {
            ctx.send(CreateReply::default().content("Not a ticket channel.").ephemeral(true))
                .await?;
            return Ok(());
        }
        if Some(user.id.get()) == service::get_opener(pool, channel_id.get()).await {
            ctx.send(
                CreateReply::default().content("Cannot remove ticket opener.").ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    }
    service::remove_user_from_ticket(ctx.http(), channel_id, user.id.get()).await?;
    let _ = channel_id
        .send_message(
            ctx.http(),
            serenity::all::CreateMessage::new().content(format!(
                "<@{}> removed <@{}>",
                ctx.author().id,
                user.id
            )),
        )
        .await;
    ctx.send(CreateReply::default().content(format!("Removed <@{}>", user.id)).ephemeral(true))
        .await?;
    Ok(())
}
