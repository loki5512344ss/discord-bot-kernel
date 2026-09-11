//! `/poll` slash command - embed with numbered options + reaction voting.

use crate::feature::Ctx;
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter, ReactionType};

const NUMBER_EMOJIS: [&str; 10] = ["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟"];

/// `/poll <question> <options>` - options separated by `;` (2-10). Adds number reactions.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn poll(
    ctx: Ctx<'_>,
    #[description = "Poll question"] question: String,
    #[description = "Options separated by ;"] options: String,
) -> anyhow::Result<()> {
    let opts: Vec<&str> = options.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
    if opts.len() < 2 || opts.len() > NUMBER_EMOJIS.len() {
        ctx.send(
            CreateReply::default()
                .content(format!("Provide 2-{} options separated by `;`.", NUMBER_EMOJIS.len()))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }
    let description = opts
        .iter()
        .enumerate()
        .map(|(i, o)| format!("{} {o}", NUMBER_EMOJIS[i]))
        .collect::<Vec<_>>()
        .join("\n");
    let embed = CreateEmbed::new()
        .title(question)
        .description(description)
        .color(0x0058_65F2)
        .footer(CreateEmbedFooter::new(format!("Poll by {}", ctx.author().name)));
    let reply = ctx.send(CreateReply::default().embed(embed)).await?;
    let msg = reply.message().await?;
    for emoji in NUMBER_EMOJIS.iter().take(opts.len()) {
        msg.react(ctx.http(), ReactionType::Unicode((*emoji).to_string())).await?;
    }
    Ok(())
}
