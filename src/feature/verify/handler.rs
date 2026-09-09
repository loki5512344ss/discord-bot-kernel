//! Reaction handler - Verified + language roles (see ARCHITECTURE.md).
//! English docs, no `sleep` without `// SAFETY:`. No dead code.

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

/// Ensures verification embed exists once on boot. Call from `ready`.
/// - Fetches last 20 messages in #verify, looks for bot's `Verification` embed.
/// - If not found, posts new embed with ✅ and reacts.
/// - Tracks only that message ID implicitly via channel check (or store in DB later).
pub async fn ensure_verify_message(
    http: &serenity::http::Http,
    channel_id: ChannelId,
) -> anyhow::Result<()> {
    let msgs = channel_id.messages(&http, serenity::all::GetMessages::new().limit(20)).await?;
    let has_verify = msgs.iter().any(|m| {
        m.author.id.get() == 1527966600173457408
            && m.embeds.iter().any(|e| e.title.as_deref() == Some("Verification"))
    });
    if has_verify {
        info!(channel = %channel_id, "verify embed already exists");
        return Ok(());
    }
    let embed = CreateEmbed::new()
        .title("Verification")
        .description(
            "Click ✅ to get access to the server.\n\nChat is locked here - reaction only.",
        )
        .color(0x002E_CC71);
    let msg = channel_id.send_message(&http, CreateMessage::new().embed(embed)).await?;
    msg.react(&http, '✅').await?;
    info!(channel = %channel_id, msg = %msg.id, "verify embed created on boot");
    Ok(())
}
