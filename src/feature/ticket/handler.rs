//! Ticket handler - SelectMenu + Modal + private channels (or Forum posts if TICKET_USE_FORUM), claim/close with transcript.
use super::service;
use crate::feature::ticket::forum;
use serenity::all::{
    AutoArchiveDuration, ChannelId, ChannelType, ComponentInteraction, Context, CreateActionRow,
    CreateChannel, CreateEmbed, CreateForumPost, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, CreateModal, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, GuildId, InputTextStyle, ModalInteraction,
    PermissionOverwrite, PermissionOverwriteType, Permissions,
};
use tracing::{info, warn};

const SUPPORT_CATEGORY: u64 = 1547213779224363008;
const TICKET_LOGS: u64 = 1509503717127356446;
/// Dedicated panel channel (#open-ticket / #🎫｜open-ticket)
const TICKET_PANEL_CHANNEL: u64 = 1509503682914549861;

/// Canonical topics - single source of truth for select menu + modal title + validation.
struct Topic {
    value: &'static str,
    label: &'static str,
    emoji: char,
    desc: &'static str,
}
const TOPICS: &[Topic] = &[
    Topic {
        value: "general",
        label: "General support",
        emoji: '🛠',
        desc: "Questions about bot / server",
    },
    Topic { value: "bug", label: "Bug report", emoji: '🐛', desc: "Report a bug or error" },
    Topic {
        value: "appeal", label: "Mute / ban appeal", emoji: '🔇', desc: "Appeal a punishment"
    },
    Topic { value: "report", label: "Report user", emoji: '😡', desc: "Report rule violation" },
    Topic {
        value: "owners",
        label: "Contact owners",
        emoji: '👑',
        desc: "Private contact with owners",
    },
];

fn topic_label(value: &str) -> Option<&'static str> {
    TOPICS.iter().find(|t| t.value == value).map(|t| t.label)
}

// ---------------------------------------------------------------------------
// Panel builders (pure, reusable)
// ---------------------------------------------------------------------------

pub fn panel_embed() -> CreateEmbed {
    CreateEmbed::new()
        .title("🎫 Support tickets")
        .description(
            "**Choose a topic below to open a private ticket.**\n\
            \n\
            • Staff will respond shortly — please describe your issue clearly.\n\
            • One open ticket per user.\n\
            • Abuse / spam → blacklist.",
        )
        .color(0x0058_65F2)
        .footer(serenity::all::CreateEmbedFooter::new(
            "Loki Dev • Select topic → fill form → ticket created",
        ))
}

pub fn panel_select_row() -> CreateActionRow {
    let options: Vec<CreateSelectMenuOption> = TOPICS
        .iter()
        .map(|t| CreateSelectMenuOption::new(t.label, t.value).description(t.desc).emoji(t.emoji))
        .collect();
    let menu = CreateSelectMenu::new("ticket_topic", CreateSelectMenuKind::String { options })
        .placeholder("Choose topic…")
        .min_values(1)
        .max_values(1);
    CreateActionRow::SelectMenu(menu)
}

/// Ensures panel exists in dedicated channel. Called on Ready + on demand from /ticket_panel.
/// Idempotent: checks last 20 messages for bot's panel (select `ticket_topic`), creates if missing.
pub async fn ensure_ticket_panel(
    http: &serenity::http::Http,
    guild_id: GuildId,
) -> anyhow::Result<()> {
    // Prefer constant id, fallback to search by name "open-ticket"
    let channel_id =
        resolve_panel_channel(http, guild_id).await.unwrap_or(ChannelId::new(TICKET_PANEL_CHANNEL));
    let msgs = channel_id.messages(http, serenity::all::GetMessages::new().limit(20)).await?;
    let has_panel = msgs.iter().any(|m| {
        m.author.id.get() == 1527966600173457408
            && m.components.iter().any(|row| {
                row.components.iter().any(|c| match c {
                    serenity::all::ActionRowComponent::SelectMenu(sm) => {
                        sm.custom_id.as_deref() == Some("ticket_topic")
                    }
                    _ => false,
                })
            })
    });
    if has_panel {
        info!(channel=%channel_id, "ticket panel already exists");
        return Ok(());
    }
    channel_id
        .send_message(
            http,
            CreateMessage::new().embed(panel_embed()).components(vec![panel_select_row()]),
        )
        .await?;
    info!(channel=%channel_id, "ticket panel created on boot");
    Ok(())
}

async fn resolve_panel_channel(
    http: &serenity::http::Http,
    guild_id: GuildId,
) -> Option<ChannelId> {
    if let Ok(channels) = guild_id.channels(http).await {
        if let Some((id, _)) = channels.iter().find(|(_, c)| c.name.contains("open-ticket")) {
            return Some(*id);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Interaction routing
// ---------------------------------------------------------------------------

/// Handles ticket buttons (claim/close) + legacy create buttons.
pub async fn on_button(ctx: Context, interaction: ComponentInteraction) {
    match interaction.data.custom_id.as_str() {
        "ticket_general" | "ticket_bug" => handle_create_legacy(ctx, interaction).await,
        "ticket_claim" => handle_claim(ctx, interaction).await,
        "ticket_close" => handle_close(ctx, interaction).await,
        _ => {}
    }
}

/// Handles select menu `ticket_topic` -> shows modal with subject/description.
pub async fn on_select(ctx: Context, interaction: ComponentInteraction) {
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
    // Pre-checks before showing modal (blacklist / existing ticket) to avoid wasted modal.
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
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            let exists = channels.values().any(|c| {
                c.name.starts_with(&format!("ticket-{}", interaction.user.name.to_lowercase()))
                    || c.name.starts_with(&format!("ticket-{}", interaction.user.id))
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

/// Handles modal submit `ticket_modal:<topic>` -> creates private ticket channel.
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
    // Basic validation mirrored from modal (discord enforces, but double-check)
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
    let Some(guild_id) = interaction.guild_id else {
        return;
    };
    let user = interaction.user.clone();
    // Re-check blacklist / duplicate (race between modal open and submit)
    if let Some(pool) = service::pool() {
        if let Some(reason) = service::is_blacklisted(pool, guild_id.get(), user.id.get()).await {
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
    // Defer ephemeral thinking before heavy channel create (avoid 3s timeout)
    let _ = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            ),
        )
        .await;
    // Use helper that supports details
    match create_ticket_channel(&ctx, guild_id, &user, topic, label, &subject, &description).await {
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

// ---------------------------------------------------------------------------
// Channel creation (shared for legacy + modal)
// ---------------------------------------------------------------------------

async fn create_ticket_channel(
    ctx: &Context,
    guild_id: GuildId,
    user: &serenity::all::User,
    topic: &str,
    label: &str,
    subject: &str,
    description: &str,
) -> anyhow::Result<ChannelId> {
    // Forum branch (if TICKET_USE_FORUM=true and forum available) - each ticket = forum post with tag
    if forum::is_enabled() {
        if let Ok(forum_id) = forum::ensure_forum_channel(&ctx.http, guild_id).await {
            if let Some(tag_id) = forum::tag_id_for_async(&ctx.http, forum_id, topic).await {
                let embed = CreateEmbed::new()
                    .title(subject)
                    .description(format!(
                        "Hello <@{}> — staff will be with you shortly.\n\n**Topic:** {label} (`{topic}`)\n**Description:**\n{description}",
                        user.id
                    ))
                    .color(0x0058_65F2)
                    .footer(serenity::all::CreateEmbedFooter::new(format!(
                        "Opened by {} • {}",
                        user.name, topic
                    )));
                // Forum post name must be 1-100 chars, use sanitized subject
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
                                    CreateEmbed::new()
                                        .title("Ticket opened (forum)")
                                        .description(format!(
                                            "<@{}> opened forum post <#{}> — **{label}**\nSubject: {subject}",
                                            user.id, thread_id
                                        ))
                                        .color(0x0058_65F2),
                                ),
                            )
                            .await;
                        info!(user=%user.name, channel=%thread_id, topic=%topic, forum=%forum_id, "ticket created via forum");
                        return Ok(thread_id);
                    }
                    Err(e) => {
                        warn!(error=%e, "forum post failed, falling back to channel");
                    }
                }
            }
        }
    }
    // Fallback: classic text channel in SUPPORT category
    let safe_user = sanitize_name(&user.name);
    let name = format!("ticket-{topic}-{safe_user}");
    let name = name.chars().take(90).collect::<String>();
    let overwrites = build_ticket_overwrites(user.id.get());
    let channel = guild_id
        .create_channel(
            &ctx.http,
            CreateChannel::new(&name)
                .kind(ChannelType::Text)
                .category(ChannelId::new(SUPPORT_CATEGORY))
                .permissions(overwrites),
        )
        .await?;
    if let Some(pool) = service::pool() {
        service::record_ticket(pool, channel.id.get(), guild_id.get(), user.id.get()).await;
    }
    let embed = CreateEmbed::new()
        .title(subject)
        .description(format!(
            "Hello <@{}> — staff will be with you shortly.\n\n**Topic:** {label} (`{topic}`)\n**Description:**\n{description}",
            user.id
        ))
        .color(0x0058_65F2)
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
                    .color(0x0058_65F2),
            ),
        )
        .await;
    info!(user=%user.name, channel=%channel.id, topic=%topic, "ticket created via modal");
    Ok(channel.id)
}

fn sanitize_name(s: &str) -> String {
    let lower = s.to_lowercase().replace(' ', "-");
    let mut out = String::new();
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let collapsed = out.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if collapsed.is_empty() { "user".into() } else { collapsed }
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

async fn handle_create_legacy(ctx: Context, interaction: ComponentInteraction) {
    // Legacy button path - create with generic subject/desc
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

async fn handle_claim(ctx: Context, interaction: ComponentInteraction) {
    let Some(guild_id) = interaction.guild_id else { return };
    let channel_id = interaction.channel_id;
    let claimer = interaction.user.clone();
    // Staff-only + not opener check (creator cannot claim own ticket)
    if let Some(pool) = service::pool() {
        if let Some(opener) = service::get_opener(pool, channel_id.get()).await {
            if opener == claimer.id.get() {
                ephemeral(&ctx, &interaction, "You cannot claim your own ticket.".into()).await;
                return;
            }
        }
    }
    // Require Staff/Owner/Moderator role
    const STAFF_ROLES: &[u64] = &[
        1547203757711040572, // Staff
        1547203762035236925, // Owner
        1547203753373859901, // Moderator
    ];
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
    // Forum threads: archive+lock instead of delete (keeps history, respects forum lifecycle)
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
        // Try delete if archive fails (fallback)
        if ctx.http.get_channel(channel_id).await.is_ok() {
            // Already archived, no delete needed; optionally delete after archive
        }
        info!(channel=%channel_id, thread=true, "ticket thread closed (archived)");
    } else {
        let _ = channel_id.delete(&ctx.http).await;
        info!(channel=%channel_id, "ticket closed");
    }
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
