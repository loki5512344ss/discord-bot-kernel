use crate::feature::Ctx;
use crate::feature::ticket::service;
use crate::util::i18n::{self, Lang};
use poise::CreateReply;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbed};

/// Panel with ticket open buttons.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn ticket_panel(ctx: Ctx<'_>) -> anyhow::Result<()> {
    let lang = if let Some(member) = ctx.author_member().await {
        let ids: Vec<u64> = member.roles.iter().map(|r| r.get()).collect();
        i18n::from_roles(&ids)
    } else {
        Lang::En
    };
    let embed = CreateEmbed::new()
        .title(i18n::t("ticket.title", lang))
        .description(i18n::t("ticket.desc", lang))
        .color(0x0058_65F2);
    let row = CreateActionRow::Buttons(vec![
        CreateButton::new("ticket_general").label("General").emoji('🛠'),
        CreateButton::new("ticket_bug").label("Bug").emoji('🐛'),
    ]);
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
