//! Reaction handler - Verified + language roles (see ARCHITECTURE.md).
//! English docs, no `sleep` without `// SAFETY:`. No dead code.

use crate::util::brand::{BOT_USER_ID, Banner};
use serenity::all::{ChannelId, Context, CreateEmbed, CreateMessage, Reaction};
use tracing::{info, warn};

/// Gives `Verified` on ✅ in #verify, and RU/EN in #roles (1527976303284715610).
pub async fn on_reaction_add(ctx: Context, reaction: Reaction) {
    if let Some(user_id) = reaction.user_id {
        if let Ok(user) = ctx.http.get_user(user_id).await {
            if user.bot {
                return;
            }
        }
    }
    let emoji = reaction.emoji.to_string();
    let channel = reaction.channel_id.get();
    const VERIFY_CHANNEL: u64 = 1509503655487737916;
    const ROLES_CHANNEL: u64 = 1527976303284715610;
    const BOT_COMMANDS: u64 = 1509503714875146350;
    const OTHER_BOT_COMMANDS: u64 = 1547203853420728352;

    // Language roles in #roles
    if channel == ROLES_CHANNEL {
        let role_id = match emoji.as_str() {
            "🇷🇺" | "🇷🇺\u{fe0f}" => Some(1547203732339433494u64),
            "🇬🇧" | "🇬🇧\u{fe0f}" | "🇬\u{fe0f}\u{200d}\u{fe0f}" => {
                Some(1547203727792807936u64)
            }
            _ => None,
        };
        if let Some(rid) = role_id {
            if let (Some(guild_id), Some(user_id)) = (reaction.guild_id, reaction.user_id) {
                if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
                    let role = serenity::all::RoleId::new(rid);
                    if !member.roles.contains(&role) {
                        let _ = member.add_role(&ctx.http, role).await;
                        info!(user = %user_id, role = %rid, "language role added");
                    }
                }
            }
        }
        return;
    }

    // Verification: ✅ in #verify (or test channels)
    if emoji != "✅" && emoji != "✅\u{fe0f}" {
        return;
    }
    if channel != VERIFY_CHANNEL && channel != BOT_COMMANDS && channel != OTHER_BOT_COMMANDS {
        return;
    }
    const VERIFIED: u64 = 1547203736676597891;
    let Some(guild_id) = reaction.guild_id else {
        return;
    };
    let Some(user_id) = reaction.user_id else {
        return;
    };
    let http = ctx.http.clone();
    match guild_id.member(&http, user_id).await {
        Ok(member) => {
            let role = serenity::all::RoleId::new(VERIFIED);
            if member.roles.contains(&role) {
                return;
            }
            match member.add_role(&http, role).await {
                Ok(()) => {
                    info!(user = %user_id, "verified via ✅");
                    // Send DM with link to #roles (ephemeral-like, visible only to user)
                    if let Ok(dm) = user_id.create_dm_channel(&http).await {
                        let _ = dm
                            .send_message(
                                &http,
                                CreateMessage::new().content(format!(
                                    "✅ You are verified! Now choose your language in <#{}>.\n🇷🇺 → Russian\n🇬🇧 → English\n\n*Bot answers will be in your selected language.*",
                                    ROLES_CHANNEL
                                )),
                            )
                            .await;
                    }
                }
                Err(e) => warn!(error = %e, user = %user_id, "failed to add Verified"),
            }
        }
        Err(e) => warn!(error = %e, "failed to fetch member for verify"),
    }
}

fn boot_embed() -> CreateEmbed {
    CreateEmbed::new()
        .title("Verification")
        .description(
            "Click ✅ to get access to the server.\n\nChat is locked here - reaction only.",
        )
        .image(Banner::Verify.url())
        .color(crate::util::brand::ACCENT)
}

/// Ensures one verification embed exists on boot. Call from `ready`.
/// - Looks for the bot's `Verification` embed in the last 20 messages of #verify.
/// - Found but outdated: re-rendered in place (reactions stay); up to date: left alone.
/// - Not found: posts a new embed and reacts with ✅.
pub async fn ensure_verify_message(
    http: &serenity::http::Http,
    channel_id: ChannelId,
) -> anyhow::Result<()> {
    let msgs = channel_id.messages(&http, serenity::all::GetMessages::new().limit(20)).await?;
    let existing = msgs.iter().find(|m| {
        m.author.id.get() == BOT_USER_ID
            && m.embeds.iter().any(|e| e.title.as_deref() == Some("Verification"))
    });
    if let Some(m) = existing {
        let edit = serenity::all::EditMessage::new().embed(boot_embed());
        if Banner::Verify.refresh(http, m, edit).await? {
            info!(channel = %channel_id, msg = %m.id, "verify embed updated in place");
        } else {
            info!(channel = %channel_id, msg = %m.id, "verify embed already up to date");
        }
        return Ok(());
    }
    let create = CreateMessage::new().add_file(Banner::Verify.attachment()).embed(boot_embed());
    let msg = channel_id.send_message(&http, create).await?;
    msg.react(&http, '✅').await?;
    info!(channel = %channel_id, msg = %msg.id, "verify embed created on boot");
    Ok(())
}
