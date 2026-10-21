//! Ticket channel/forums creation (shared for modal + legacy).

use super::{SUPPORT_CATEGORY, TICKET_LOGS, build_ticket_overwrites, ephemeral, sanitize_name};
use crate::feature::ticket::service;
use crate::feature::ticket::service::forum;
use serenity::all::{
    AutoArchiveDuration, ChannelId, ChannelType, Context, CreateChannel, CreateEmbed,
    CreateForumPost, CreateMessage, GuildId,
};
use tracing::{info, warn};

pub async fn create_ticket_channel(
    ctx: &Context,
    guild_id: GuildId,
    user: &serenity::all::User,
    topic: &str,
    label: &str,
    subject: &str,
    description: &str,
) -> anyhow::Result<ChannelId> {
    // Forum branch (if TICKET_USE_FORUM=true)
    if forum::is_enabled() {
        if let Ok(forum_id) = forum::ensure_forum_channel(&ctx.http, guild_id).await {
            if let Some(tag_id) = forum::tag_id_for_async(&ctx.http, forum_id, topic).await {
                let embed = CreateEmbed::new()
                    .title(subject)
                    .description(format!(
                        "Hello <@{}> — staff will be with you shortly.\n\n**Topic:** {label} (`{topic}`)\n**Description:**\n{description}",
                        user.id
                    ))
                    .color(crate::util::brand::ACCENT)
                    .footer(serenity::all::CreateEmbedFooter::new(format!("Opened by {} • {}", user.name, topic)));
                let post_name = subject.chars().take(90).collect::<String>();
                let post_name = if post_name.trim().len() < 2 {
                    format!("ticket-{topic}-{}", sanitize_name(&user.name))
                } else {
                    post_name
                };
                match forum_id
                    .create_forum_post(
                        &ctx.http,
                        CreateForumPost::new(post_name.clone(), CreateMessage::new().embed(embed))
                            .add_applied_tag(tag_id)
                            .auto_archive_duration(AutoArchiveDuration::OneWeek),
                    )
                    .await
                {
                    Ok(thread) => {
                        let thread_id = thread.id;
                        if let Some(pool) = service::pool() {
                            service::record_ticket(
                                pool,
                                thread_id.get(),
                                guild_id.get(),
                                user.id.get(),
                            )
                            .await;
                        }
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
                        let _ = thread_id
                            .send_message(
                                &ctx.http,
                                CreateMessage::new()
                                    .content(format!("<@{}> — staff control", user.id))
                                    .components(vec![row]),
                            )
                            .await;
                        let _ = ChannelId::new(TICKET_LOGS)
                            .send_message(
                                &ctx.http,
                                CreateMessage::new().embed(
                                    CreateEmbed::new().title("Ticket opened (forum)").description(format!("<@{}> opened forum post <#{}> — **{label}**\nSubject: {subject}", user.id, thread_id)).color(crate::util::brand::ACCENT),
                                ),
                            )
                            .await;
                        info!(user=%user.name, channel=%thread_id, topic=%topic, forum=%forum_id, "ticket created via forum");
                        return Ok(thread_id);
                    }
                    Err(e) => warn!(error=%e, "forum post failed, falling back to channel"),
                }
            }
        }
    }
    // Fallback: classic text channel in SUPPORT category
    let safe_user = sanitize_name(&user.name);
    let name = format!("ticket-{topic}-{safe_user}").chars().take(90).collect::<String>();
    let overwrites = build_ticket_overwrites(user.id.get());
    let channel = guild_id
        .create_channel(
            &ctx.http,
            CreateChannel::new(&name)
                .kind(ChannelType::Text)
                .category(ChannelId::new(SUPPORT_CATEGORY))
                .permissions(overwrites)
                .rate_limit_per_user(5),
        )
        .await?;
    if let Some(pool) = service::pool() {
        service::record_ticket(pool, channel.id.get(), guild_id.get(), user.id.get()).await;
    }
    let embed = CreateEmbed::new()
        .title(subject)
        .description(format!("Hello <@{}> — staff will be with you shortly.\n\n**Topic:** {label} (`{topic}`)\n**Description:**\n{description}", user.id))
        .color(crate::util::brand::ACCENT)
        .footer(serenity::all::CreateEmbedFooter::new(format!("Opened by {} • {}", user.name, topic)));
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
    let _ = ChannelId::new(TICKET_LOGS)
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(
                CreateEmbed::new()
                    .title("Ticket opened")
                    .description(format!(
                        "<@{}> opened <#{}> — **{label}**\nSubject: {subject}",
                        user.id, channel.id
                    ))
                    .color(crate::util::brand::ACCENT),
            ),
        )
        .await;
    info!(user=%user.name, channel=%channel.id, topic=%topic, "ticket created via modal");
    Ok(channel.id)
}

pub async fn handle_create_legacy(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    let Some(guild_id) = interaction.guild_id else { return };
    let user = interaction.user.clone();
    let custom = interaction.data.custom_id.clone();
    if let Some(pool) = service::pool() {
        if let Some(reason) = service::is_blacklisted(pool, guild_id.get(), user.id.get()).await {
            ephemeral(&ctx, &interaction, format!("You are blacklisted: {reason}")).await;
            return;
        }
    }
    let (topic, label) =
        if custom == "ticket_bug" { ("bug", "Bug report") } else { ("general", "General support") };
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
    match create_ticket_channel(
        &ctx,
        guild_id,
        &user,
        topic,
        label,
        &format!("Ticket - {label}"),
        "Opened via legacy button.",
    )
    .await
    {
        Ok(cid) => ephemeral(&ctx, &interaction, format!("Ticket created: <#{cid}>")).await,
        Err(e) => {
            warn!(error=%e, "failed to create legacy ticket");
            ephemeral(&ctx, &interaction, "Failed to create ticket.".into()).await;
        }
    }
}
