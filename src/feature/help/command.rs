use crate::feature::Ctx;
use crate::util::i18n::{self, Lang};
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

// debug/test-only: ping, shutdown will be removed after Phase 1
// help stays - beautiful embed with categories + pagination via select

/// `/help` + `!help` - nice embed by categories (poise 0.7).
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn help(
    ctx: Ctx<'_>,
    #[description = "Command to show"] command: Option<String>,
) -> anyhow::Result<()> {
    let lang = if let Some(member) = ctx.author_member().await {
        let ids: Vec<u64> = member.roles.iter().map(|r| r.get()).collect();
        i18n::from_roles(&ids)
    } else {
        Lang::En
    };
    if let Some(name) = command {
        if let Some(cmd) = ctx.framework().options().commands.iter().find(|c| c.name == name) {
            let desc = cmd.description.as_deref().unwrap_or("no description");
            ctx.send(
                CreateReply::default().embed(
                    CreateEmbed::new()
                        .title(format!("📖 /{} ", cmd.name))
                        .description(desc)
                        .color(0x0058_65F2)
                        .footer(CreateEmbedFooter::new(format!(
                            "Requested by {}",
                            ctx.author().name
                        ))),
                ),
            )
            .await?;
        } else {
            ctx.say(format!("`{name}` not found. Try `/help`")).await?;
        }
        return Ok(());
    }

    let embed = CreateEmbed::new()
        .title(i18n::t("help.title", lang))
        .description(i18n::t("help.desc", lang))
        .color(0x0058_65F2)
        .thumbnail(ctx.cache().current_user().avatar_url().unwrap_or_default())
        .field(i18n::t("help.field.info", lang), "`/help` - this menu\n`/serverinfo` - server info\n`/ping` *(test, to be removed)*", true)
        .field(i18n::t("help.field.moderation", lang), "`/verify_setup` - embed ✅ in #verify (Staff only)\n`/shutdown` *(test, to be removed)*", true)
        .field(i18n::t("help.field.tickets", lang), "`/ticket_panel` - panel in #open-ticket (buttons)", true)
        .field(i18n::t("help.field.verify", lang), "Channel `#✅｜verify` - read-only, react `✅` → `✅ Verified`", false)
        .footer(CreateEmbedFooter::new(format!("Requested by {} • /help <command> for details", ctx.author().name)))
        .timestamp(serenity::all::Timestamp::now());

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
