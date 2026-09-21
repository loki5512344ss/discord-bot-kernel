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
                    .color(0x0057_F387),
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
                    .color(0x0057_F387),
            ),
        )
        .await;
    info!(channel=%channel_id, claimer=%claimer.id, "ticket claimed");
}

pub async fn handle_close(ctx: Context, interaction: serenity::all::ComponentInteraction) {
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
    tokio::time::sleep(std::time::Duration::from_secs(2)).await; // SAFETY: small delay to let log flush
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
        info!(channel=%channel_id, thread=true, "ticket thread closed (archived)");
    } else {
        let _ = channel_id.delete(&ctx.http).await;
        info!(channel=%channel_id, "ticket closed");
    }
}
