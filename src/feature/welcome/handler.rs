//! Welcome handler - posts random greeting in #welcome (1509503664438378596).
//! English, no `sleep` without `// SAFETY:`.

use serenity::all::{ChannelId, Context, CreateEmbed, CreateMessage, Member};
use tracing::{info, warn};

use super::greetings::random_greeting;

const WELCOME_CHANNEL: ChannelId = ChannelId::new(1509503664438378596);

/// Called on `GuildMemberAdd` (see `core::bot`).
pub async fn on_member_join(ctx: Context, member: Member) {
    let name = member.display_name().to_string();
    let greeting = random_greeting(&name);
    let embed = CreateEmbed::new()
        .title("Welcome! 👋")
        .description(greeting)
        .color(0x5865F2)
        .thumbnail(member.user.avatar_url().unwrap_or_default())
        .footer(serenity::all::CreateEmbedFooter::new(
            "Loki Dev • Verify in #verify • Pick language in #roles",
        ));
    let res = WELCOME_CHANNEL
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(embed).content(format!("<@{}>", member.user.id)),
        )
        .await;
    match res {
        Ok(_) => info!(user = %member.user.name, "welcome sent"),
        Err(e) => warn!(error = %e, "failed to send welcome"),
    }
    // Also trigger language picker (defer to lang handler - already wired in bot.rs)
}
