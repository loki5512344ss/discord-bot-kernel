use crate::feature::Ctx;
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

/// Shows help with interactive category selector.
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn help(
    ctx: Ctx<'_>,
    #[description = "Command to show"] command: Option<String>,
) -> anyhow::Result<()> {
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
    let prefix = ctx.data().cfg.prefix.clone();
    let guild_icon = ctx
        .guild_id()
        .and_then(|_| ctx.cache().guild(ctx.guild_id()?).map(|g| g.icon_url().unwrap_or_default()));
    let embed = crate::feature::help::handler::main_embed(&prefix, &ctx.author().name, guild_icon);
    let row = crate::feature::help::handler::selector_row();
    ctx.send(CreateReply::default().embed(embed).components(vec![row])).await?;
    Ok(())
}
