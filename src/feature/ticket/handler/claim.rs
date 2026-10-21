//! Ticket claim/close (staff-only claim).

use crate::feature::ticket::service;
use serenity::all::{ChannelId, Context, CreateEmbed, CreateMessage};
use tracing::info;

async fn ephemeral(ctx: &Context, i: &serenity::all::ComponentInteraction, content: String) {
    let _ = i
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await;
}

pub async fn handle_claim(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    let Some(guild_id) = interaction.guild_id else { return };
    let channel_id = interaction.channel_id;
    let claimer = interaction.user.clone();
    if let Some(pool) = service::pool() {
        if let Some(opener) = service::get_opener(pool, channel_id.get()).await {
            if opener == claimer.id.get() {
                ephemeral(&ctx, &interaction, "You cannot claim your own ticket.".into()).await;
                return;
            }
        }
    }
    const STAFF_ROLES: &[u64] = &[1547203757711040572, 1547203762035236925, 1547203753373859901];
    let is_staff = match guild_id.member(&ctx.http, claimer.id).await {
        Ok(member) => member.roles.iter().any(|r| STAFF_ROLES.contains(&r.get())),
        Err(_) => false,
    };
    if !is_staff {
        ephemeral(&ctx, &interaction, "Only staff can claim tickets.".into()).await;
        return;
    }
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
                    .color(crate::util::brand::OK),
            ),
        )
        .await;
    let _ = ChannelId::new(super::TICKET_LOGS)
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("Ticket claimed")
                    .description(format!("<@{}> claimed ticket <#{}>", claimer.id, channel_id))
                    .color(crate::util::brand::OK),
            ),
        )
        .await;
    info!(channel=%channel_id, claimer=%claimer.id, "ticket claimed");
}

pub async fn handle_close(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    let modal =
        serenity::all::CreateModal::new("ticket_close_modal", "Close ticket").components(vec![
            serenity::all::CreateActionRow::InputText(
                serenity::all::CreateInputText::new(
                    serenity::all::InputTextStyle::Paragraph,
                    "Reason",
                    "close_reason",
                )
                .placeholder("Why are you closing? (10-500 chars)")
                .min_length(10)
                .max_length(500)
                .required(true),
            ),
        ]);
    let _ = interaction
        .create_response(&ctx.http, serenity::all::CreateInteractionResponse::Modal(modal))
        .await;
}
pub async fn handle_close_modal(ctx: Context, interaction: serenity::all::ModalInteraction) {
    if interaction.data.custom_id != "ticket_close_modal" {
        return;
    }
    let reason = interaction
        .data
        .components
        .iter()
        .flat_map(|r| &r.components)
        .find_map(|c| match c {
            serenity::all::ActionRowComponent::InputText(it) if it.custom_id == "close_reason" => {
                it.value.clone()
            }
            _ => None,
        })
        .unwrap_or_default();
    if reason.trim().len() < 10 {
        let _ = interaction
            .create_response(
                &ctx.http,
                serenity::all::CreateInteractionResponse::Message(
                    serenity::all::CreateInteractionResponseMessage::new()
                        .content("Reason 10-500 chars required.")
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    }
    let channel_id = interaction.channel_id;
    let user = interaction.user.clone();
    let _ = interaction
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Defer(
                serenity::all::CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await;
    let transcript = service::fetch_transcript(&ctx.http, channel_id).await;
    let haste_url = service::upload_hastebin(&transcript).await;
    service::deliver_transcript(
        &ctx.http,
        channel_id,
        user.id.get(),
        format!("{transcript}\n\nClose reason: {reason}"),
        haste_url.clone(),
    )
    .await;
    let opener_for_cooldown = if let Some(pool) = service::pool() {
        service::get_opener(pool, channel_id.get()).await
    } else {
        None
    };
    if let Some(pool) = service::pool() {
        service::cleanup_ticket(pool, channel_id.get()).await;
        if let Some(opener) = opener_for_cooldown {
            if let Some(gid) = interaction.guild_id {
                service::set_cooldown(pool, gid.get(), opener, 300).await;
            }
        }
    }
    let control_row = serenity::all::CreateActionRow::Buttons(vec![
        serenity::all::CreateButton::new("ticket_reopen")
            .label("Reopen")
            .style(serenity::all::ButtonStyle::Secondary),
        serenity::all::CreateButton::new("ticket_transcript")
            .label("Save")
            .style(serenity::all::ButtonStyle::Secondary),
        serenity::all::CreateButton::new("ticket_delete")
            .label("Delete")
            .style(serenity::all::ButtonStyle::Danger),
    ]);
    let embed = CreateEmbed::new()
        .title("Ticket closed")
        .description(format!("Closed by <@{}>\nReason: {reason}", user.id))
        .color(crate::util::brand::DANGER)
        .footer(serenity::all::CreateEmbedFooter::new("Control panel below"));
    if let Some(url) = haste_url.clone() {
        let link_row = serenity::all::CreateActionRow::Buttons(vec![
            serenity::all::CreateButton::new_link(url).label("View transcript"),
        ]);
        let _ = channel_id
            .send_message(
                &ctx.http,
                CreateMessage::new().embed(embed).components(vec![control_row, link_row]),
            )
            .await;
    } else {
        let _ = channel_id
            .send_message(
                &ctx.http,
                CreateMessage::new().embed(embed).components(vec![control_row]),
            )
            .await;
    }
    let _ = ChannelId::new(super::TICKET_LOGS)
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("Ticket closed")
                    .description(format!(
                        "<@{}> closed <#{}>\nReason: {reason}",
                        user.id, channel_id
                    ))
                    .color(crate::util::brand::DANGER),
            ),
        )
        .await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let _ = interaction
        .create_followup(
            &ctx.http,
            serenity::all::CreateInteractionResponseFollowup::new()
                .content(format!("Ticket <#{channel_id}> closed."))
                .ephemeral(true),
        )
        .await;
    let is_thread = ctx
        .http
        .get_channel(channel_id)
        .await
        .map(|c| matches!(c, serenity::all::Channel::Guild(gc) if gc.thread_metadata.is_some()))
        .unwrap_or(false);
    if is_thread {
        let _ = channel_id
            .edit_thread(&ctx.http, serenity::all::EditThread::new().archived(true).locked(true))
            .await;
    }
    info!(channel=%channel_id, closer=%user.id, "ticket closed with reason");
}
pub async fn handle_reopen(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    let channel_id = interaction.channel_id;
    let is_thread = ctx
        .http
        .get_channel(channel_id)
        .await
        .map(|c| matches!(c, serenity::all::Channel::Guild(gc) if gc.thread_metadata.is_some()))
        .unwrap_or(false);
    if is_thread {
        let _ = channel_id
            .edit_thread(&ctx.http, serenity::all::EditThread::new().archived(false).locked(false))
            .await;
    }
    ephemeral(&ctx, &interaction, "Ticket reopened.".into()).await;
    let _ = channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("Ticket reopened")
                    .description(format!("Reopened by <@{}>", interaction.user.id))
                    .color(crate::util::brand::OK),
            ),
        )
        .await;
}
pub async fn handle_transcript(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    let channel_id = interaction.channel_id;
    ephemeral(&ctx, &interaction, "Generating transcript...".into()).await;
    let transcript = service::fetch_transcript(&ctx.http, channel_id).await;
    if let Some(url) = service::upload_hastebin(&transcript).await {
        let _ = channel_id
            .send_message(&ctx.http, CreateMessage::new().content(format!("Transcript: {url}")))
            .await;
    } else {
        let data =
            serenity::all::CreateAttachment::bytes(transcript.into_bytes(), "transcript.txt");
        let _ = channel_id.send_message(&ctx.http, CreateMessage::new().add_file(data)).await;
    }
}
pub async fn handle_delete(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    let channel_id = interaction.channel_id;
    ephemeral(&ctx, &interaction, "Deleting ticket...".into()).await;
    if let Some(pool) = service::pool() {
        service::cleanup_ticket(pool, channel_id.get()).await;
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let _ = channel_id.delete(&ctx.http).await;
    info!(channel=%channel_id, "ticket deleted via panel");
}
