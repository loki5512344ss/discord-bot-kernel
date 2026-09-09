//! Ticket handler - creates private channels, claim and close with transcript.
use super::service;
use serenity::all::{
    ChannelId, ChannelType, ComponentInteraction, Context, CreateChannel, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    PermissionOverwrite, PermissionOverwriteType, Permissions,
};
use tracing::{info, warn};
const SUPPORT_CATEGORY: u64 = 1547213779224363008;
const TICKET_LOGS: u64 = 1509503717127356446;
/// Handles ticket buttons.
pub async fn on_button(ctx: Context, interaction: ComponentInteraction) {
    match interaction.data.custom_id.as_str() {
        "ticket_general" | "ticket_bug" => handle_create(ctx, interaction).await,
        "ticket_claim" => handle_claim(ctx, interaction).await,
        "ticket_close" => handle_close(ctx, interaction).await,
        _ => {}
    }
}
async fn ephemeral(ctx: &Context, i: &ComponentInteraction, content: String) {
    let _ = i
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(content).ephemeral(true),
            ),
        )
        .await;
}
async fn handle_create(ctx: Context, interaction: ComponentInteraction) {
    let Some(guild_id) = interaction.guild_id else { return };
    let user = interaction.user.clone();
    let custom = interaction.data.custom_id.clone();
    if let Some(pool) = service::pool() {
        if let Some(reason) = service::is_blacklisted(pool, guild_id.get(), user.id.get()).await {
            ephemeral(&ctx, &interaction, format!("You are blacklisted: {reason}")).await;
            return;
        }
    }
    let category = if custom == "ticket_bug" { "bug" } else { "general" };
    if let Ok(channels) = guild_id.channels(&ctx.http).await {
        let exists = channels.values().any(|c| {
            c.name.starts_with(&format!("ticket-{}", user.name.to_lowercase()))
                || c.name.starts_with(&format!("ticket-{}", user.id))
        });
        if exists {
            ephemeral(&ctx, &interaction, "You already have an open ticket.".into()).await;
            return;
        }
    }
    let name = format!("ticket-{}-{}", category, user.name.to_lowercase().replace(' ', "-"));
    let overwrites = build_ticket_overwrites(user.id.get());
    let channel = match guild_id
        .create_channel(
            &ctx.http,
            CreateChannel::new(&name)
                .kind(ChannelType::Text)
                .category(ChannelId::new(SUPPORT_CATEGORY))
                .permissions(overwrites),
        )
        .await
    {
        Ok(ch) => ch,
        Err(e) => {
            warn!(error=%e, "failed to create ticket");
            ephemeral(&ctx, &interaction, "Failed to create ticket.".into()).await;
            return;
        }
    };
    if let Some(pool) = service::pool() {
        service::record_ticket(pool, channel.id.get(), guild_id.get(), user.id.get()).await;
    }
    let embed = CreateEmbed::new()
        .title(format!("Ticket - {category}"))
        .description(format!(
            "Hello <@{}>, staff will be with you shortly.\n\nCategory: **{category}**",
            user.id
        ))
        .color(0x0058_65F2);
    let row = serenity::all::CreateActionRow::Buttons(vec![
        serenity::all::CreateButton::new("ticket_claim")
            .label("Claim")
            .style(serenity::all::ButtonStyle::Primary)
            .emoji('🙋'),
        serenity::all::CreateButton::new("ticket_close")
            .label("Close")
            .style(serenity::all::ButtonStyle::Danger)
            .emoji('🔒'),
    ]);
    let _ = channel
        .send_message(&ctx.http, CreateMessage::new().embed(embed).components(vec![row]))
        .await;
    ephemeral(&ctx, &interaction, format!("Ticket created: <#{}>", channel.id)).await;
    info!(user=%user.name, channel=%channel.id, "ticket created");
}
async fn handle_claim(ctx: Context, interaction: ComponentInteraction) {
    let Some(guild_id) = interaction.guild_id else { return };
    let channel_id = interaction.channel_id;
    let claimer = interaction.user.clone();
    if let Some(pool) = service::pool() {
        if let Some(existing) = service::get_claim(pool, channel_id.get()).await {
            ephemeral(&ctx, &interaction, format!("Already claimed by <@{existing}>")).await;
            return;
        }
        if !service::try_claim(pool, channel_id.get(), guild_id.get(), claimer.id.get()).await {
            ephemeral(&ctx, &interaction, "Claim failed, already claimed.".into()).await;
            return;
        }
    }
    ephemeral(&ctx, &interaction, format!("Ticket claimed by <@{}>", claimer.id)).await;
    let _ = channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("Ticket claimed")
                    .description(format!("Claimed by <@{}>", claimer.id))
                    .footer(serenity::all::CreateEmbedFooter::new(format!(
                        "Claimed by {}",
                        claimer.name
                    )))
                    .color(0x0057_F387),
            ),
        )
        .await;
    let _ = ChannelId::new(TICKET_LOGS)
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("Ticket claimed")
                    .description(format!("<@{}> claimed ticket <#{}>", claimer.id, channel_id))
                    .color(0x0057_F387),
            ),
        )
        .await;
    info!(channel=%channel_id, claimer=%claimer.id, "ticket claimed");
}
async fn handle_close(ctx: Context, interaction: ComponentInteraction) {
    let channel_id = interaction.channel_id;
    let user = interaction.user.clone();
    ephemeral(&ctx, &interaction, "Closing ticket... transcript will be saved.".into()).await;
    let transcript = service::fetch_transcript(&ctx.http, channel_id).await;
    let haste_url = service::upload_hastebin(&transcript).await;
    service::deliver_transcript(
        &ctx.http,
        channel_id,
        user.id.get(),
        transcript.clone(),
        haste_url.clone(),
    )
    .await;
    if let Some(pool) = service::pool() {
        service::cleanup_ticket(pool, channel_id.get()).await;
    }
    tokio::time::sleep(std::time::Duration::from_secs(2)).await; // SAFETY: small delay to let log flush, not in command path, jitter not needed
    let _ = channel_id.delete(&ctx.http).await;
    info!(channel=%channel_id, "ticket closed");
}
fn allow_view(id: serenity::all::RoleId) -> PermissionOverwrite {
    PermissionOverwrite {
        allow: Permissions::VIEW_CHANNEL
            | Permissions::SEND_MESSAGES
            | Permissions::READ_MESSAGE_HISTORY,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role(id),
    }
}
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
        allow_view(staff),
        allow_view(owner),
        allow_view(moderator),
    ]
}
