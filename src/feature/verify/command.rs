use crate::feature::Ctx;
use crate::util::i18n::{self, Lang};
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateMessage};

/// Manual resend of verify embed (auto on boot)
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn verify_setup(ctx: Ctx<'_>) -> anyhow::Result<()> {
    let lang = if let Some(member) = ctx.author_member().await {
        let ids: Vec<u64> = member.roles.iter().map(|r| r.get()).collect();
        i18n::from_roles(&ids)
    } else {
        Lang::En
    };
    tracing::info!(lang = lang.code(), "verify_setup invoked");
    let channel = ctx.channel_id();
    let embed = CreateEmbed::new()
        .title(i18n::t("verify.title", lang))
        .description(i18n::t("verify.desc", lang))
        .image(crate::util::brand::Banner::Verify.url())
        .color(crate::util::brand::ACCENT);
    let msg = channel
        .send_message(
            ctx.http(),
            CreateMessage::new()
                .add_file(crate::util::brand::Banner::Verify.attachment())
                .embed(embed),
        )
        .await?;
    msg.react(ctx.http(), '✅').await?;
    ctx.send(CreateReply::default().content("Verification message posted.").ephemeral(true))
        .await?;
    Ok(())
}
