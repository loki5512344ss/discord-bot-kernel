use crate::feature::Ctx;
use crate::util::i18n::{self, Lang};
use poise::CreateReply;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbed};

/// /ticket-panel - panel with buttons in #open-ticket.
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
