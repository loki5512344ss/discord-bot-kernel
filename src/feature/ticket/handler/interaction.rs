//! Ticket interaction - select menu → modal, modal submit → channel create (defer fast).

use super::{create, topic_label};
use crate::feature::ticket::service;
use serenity::all::{
    Context, CreateActionRow, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateModal, InputTextStyle, ModalInteraction,
};
use tracing::warn;

/// Handles select menu `ticket_topic` -> shows modal.
pub async fn on_select(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    if interaction.data.custom_id != "ticket_topic" {
        return;
    }
    let topic = match &interaction.data.kind {
        serenity::all::ComponentInteractionDataKind::StringSelect { values } => {
            values.first().cloned().unwrap_or_default()
        }
        _ => return,
    };
    if topic.is_empty() {
        return;
    }
    let Some(label) = topic_label(&topic) else {
        let _ = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(format!("Unknown topic: {topic}"))
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    };
    // Fast pre-check: only blacklist (DB, ~5ms). Duplicate moved to modal submit.
    if let Some(guild_id) = interaction.guild_id {
        if let Some(pool) = service::pool() {
            if let Some(reason) =
                service::is_blacklisted(pool, guild_id.get(), interaction.user.id.get()).await
            {
                let _ = interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(format!("You are blacklisted: {reason}"))
                                .ephemeral(true),
                        ),
                    )
                    .await;
                return;
            }
        }
    }
    let modal = CreateModal::new(format!("ticket_modal:{topic}"), format!("New ticket • {label}"))
        .components(vec![
            CreateActionRow::InputText(
                serenity::all::CreateInputText::new(InputTextStyle::Short, "Subject", "subject")
                    .placeholder("Brief title (3-100 chars)")
                    .min_length(3)
                    .max_length(100)
                    .required(true),
            ),
            CreateActionRow::InputText(
                serenity::all::CreateInputText::new(
                    InputTextStyle::Paragraph,
                    "Description",
                    "description",
                )
                .placeholder("Describe your issue in detail (10-1000 chars)")
                .min_length(10)
                .max_length(1000)
                .required(true),
            ),
        ]);
    let _ = interaction.create_response(&ctx.http, CreateInteractionResponse::Modal(modal)).await;
}

/// Handles modal submit `ticket_modal:<topic>` -> creates ticket.
pub async fn on_modal(ctx: Context, interaction: ModalInteraction) {
    let Some(topic) = interaction.data.custom_id.strip_prefix("ticket_modal:") else {
        return;
    };
    let Some(label) = topic_label(topic) else {
        let _ = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Unknown ticket topic.")
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    };
    let subject =
        extract_modal_field(&interaction, "subject").unwrap_or_else(|| "No subject".into());
    let description = extract_modal_field(&interaction, "description").unwrap_or_default();
    if subject.trim().len() < 3 || description.trim().len() < 10 {
        let _ = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Subject ≥3 and description ≥10 chars required.")
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    }
    let Some(guild_id) = interaction.guild_id else { return };
    let user = interaction.user.clone();
    // Defer immediately — channel creation is slow (1-2s)
    let _ = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await;
    if let Some(pool) = service::pool() {
        if let Some(reason) = service::is_blacklisted(pool, guild_id.get(), user.id.get()).await {
            let _ = interaction
                .create_followup(
                    &ctx.http,
                    serenity::all::CreateInteractionResponseFollowup::new()
                        .content(format!("You are blacklisted: {reason}"))
                        .ephemeral(true),
                )
                .await;
            return;
        }
        if let Some(rem) = service::is_on_cooldown(pool, guild_id.get(), user.id.get()).await {
            let _ = interaction
                .create_followup(
                    &ctx.http,
                    serenity::all::CreateInteractionResponseFollowup::new()
                        .content(format!(
                            "Cooldown: wait {}s before new ticket (5m after close).",
                            rem
                        ))
                        .ephemeral(true),
                )
                .await;
            return;
        }
    }
    if let Ok(channels) = guild_id.channels(&ctx.http).await {
        let exists = channels.values().any(|c| {
            c.name.starts_with(&format!("ticket-{}", user.name.to_lowercase()))
                || c.name.starts_with(&format!("ticket-{}", user.id))
        });
        if exists {
            let _ = interaction
                .create_followup(
                    &ctx.http,
                    serenity::all::CreateInteractionResponseFollowup::new()
                        .content("You already have an open ticket.")
                        .ephemeral(true),
                )
                .await;
            return;
        }
    }
    match create::create_ticket_channel(&ctx, guild_id, &user, topic, label, &subject, &description)
        .await
    {
        Ok(channel_id) => {
            let _ = interaction
                .create_followup(
                    &ctx.http,
                    serenity::all::CreateInteractionResponseFollowup::new()
                        .content(format!("Ticket created: <#{channel_id}>"))
                        .ephemeral(true),
                )
                .await;
        }
        Err(e) => {
            warn!(error=%e, "failed to create ticket via modal");
            let _ = interaction
                .create_followup(
                    &ctx.http,
                    serenity::all::CreateInteractionResponseFollowup::new()
                        .content("Failed to create ticket. Try again later.")
                        .ephemeral(true),
                )
                .await;
        }
    }
}

fn extract_modal_field(modal: &ModalInteraction, custom_id: &str) -> Option<String> {
    for row in &modal.data.components {
        for comp in &row.components {
            if let serenity::all::ActionRowComponent::InputText(it) = comp {
                if it.custom_id == custom_id {
                    return it.value.clone();
                }
            }
        }
    }
    None
}
