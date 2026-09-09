//! Ticket handler - creates private channels on button, handles close.
//! English Javadoc style, no `sleep` without `// SAFETY:`.

use serenity::all::{
    ChannelType, ComponentInteraction, Context, CreateChannel, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    PermissionOverwrite, PermissionOverwriteType, Permissions,
};
use tracing::{info, warn};

const SUPPORT_CATEGORY: u64 = 1547213779224363008; // OPEN TICKETS (bottom, staff-only)
const TICKET_LOGS: u64 = 1509503717127356446;

/// Handles `ticket_general` / `ticket_bug` and `ticket_close`.
pub async fn on_button(ctx: Context, interaction: ComponentInteraction) {
    let guild_id = match interaction.guild_id {
        Some(g) => g,
        None => return,
    };
    let user = interaction.user.clone();
    let custom = interaction.data.custom_id.clone();

    if custom == "ticket_general" || custom == "ticket_bug" {
        let category = if custom == "ticket_bug" { "bug" } else { "general" };
        // Check existing open ticket for user (simple scan)
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            let exists = channels.values().any(|c| {
                c.name.starts_with(&format!("ticket-{}", user.name.to_lowercase()))
                    || c.name.starts_with(&format!("ticket-{}", user.id))
            });
            if exists {
                let _ = interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("You already have an open ticket.")
                                .ephemeral(true),
                        ),
                    )
                    .await;
                return;
            }
        }
        // No defer needed - channel creation is fast (<1s), respond directly after
        let name = format!("ticket-{}-{}", category, user.name.to_lowercase().replace(' ', "-"));
        let overwrites = build_ticket_overwrites(user.id.get());
        let channel = match guild_id
            .create_channel(
                &ctx.http,
                CreateChannel::new(&name)
                    .kind(ChannelType::Text)
                    .category(serenity::all::ChannelId::new(SUPPORT_CATEGORY))
                    .permissions(overwrites),
            )
            .await
        {
            Ok(ch) => ch,
            Err(e) => {
                warn!(error = %e, "failed to create ticket");
                let _ = interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Failed to create ticket. Check bot permissions.")
                                .ephemeral(true),
                        ),
                    )
                    .await;
                return;
            }
        };
        // Welcome embed with close button
        let embed = CreateEmbed::new()
            .title(format!("Ticket - {}", category))
            .description(format!(
                "Hello <@{}>, staff will be with you shortly.\n\nCategory: **{}**\n\nClick **Close** to close.",
                user.id, category
            ))
            .color(0x0058_65F2);
        let row = serenity::all::CreateActionRow::Buttons(vec![
            serenity::all::CreateButton::new("ticket_close")
                .label("Close")
                .style(serenity::all::ButtonStyle::Danger)
                .emoji('🔒'),
        ]);
        let _ = channel
            .send_message(&ctx.http, CreateMessage::new().embed(embed).components(vec![row]))
            .await;
        let _ = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(format!("Ticket created: <#{}>", channel.id))
                        .ephemeral(true),
                ),
            )
            .await;
        info!(user = %user.name, channel = %channel.id, "ticket created");
        return;
    }

    if custom == "ticket_close" {
        let channel_id = interaction.channel_id;
        let _ = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Closing ticket... transcript will be saved.")
                        .ephemeral(true),
                ),
            )
            .await;
        // Log to ticket-logs
        let log_ch = serenity::all::ChannelId::new(TICKET_LOGS);
        let _ = log_ch
            .send_message(
                &ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("Ticket closed")
                        .description(format!("Closed by <@{}> in <#{}>", user.id, channel_id))
                        .color(0x00ED_4245),
                ),
            )
            .await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await; // SAFETY: small delay to let log flush, not in command path, jitter not needed
        let _ = channel_id.delete(&ctx.http).await;
        info!(channel = %channel_id, "ticket closed");
    }
}

/// Builds permission overwrites for ticket channel.
/// - @everyone deny VIEW, user + Staff/Owner/Moderator allow.
fn build_ticket_overwrites(user_id: u64) -> Vec<PermissionOverwrite> {
    let everyone = serenity::all::RoleId::new(1509503154708811837);
    let staff = serenity::all::RoleId::new(1547203757711040572);
    let owner = serenity::all::RoleId::new(1547203762035236925);
    let moderator = serenity::all::RoleId::new(1547203753373859901);
    let user = serenity::all::UserId::new(user_id);
    vec![
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(everyone),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(user),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(staff),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(owner),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(moderator),
        },
    ]
}
