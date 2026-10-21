//! Ticket panel lifecycle on boot: find the bot's panel, refresh it in place or post it.

use crate::feature::ticket::handler::{panel_embed, panel_select_row};
use crate::util::brand::{BOT_USER_ID, Banner};
use serenity::all::{
    ActionRowComponent, ChannelId, CreateMessage, EditMessage, GetMessages, GuildId, Http, Message,
};
use tracing::info;

const TICKET_PANEL_CHANNEL: u64 = 1509503682914549861;

fn is_panel(m: &Message) -> bool {
    let is_topic_menu = |c: &ActionRowComponent| match c {
        ActionRowComponent::SelectMenu(sm) => sm.custom_id.as_deref() == Some("ticket_topic"),
        _ => false,
    };
    m.author.id.get() == BOT_USER_ID
        && m.components.iter().flat_map(|row| &row.components).any(is_topic_menu)
}

/// Ensures exactly one panel exists; an outdated one is re-rendered in place. Idempotent.
pub async fn ensure_ticket_panel(http: &Http, guild_id: GuildId) -> anyhow::Result<()> {
    let channel_id =
        resolve_panel_channel(http, guild_id).await.unwrap_or(ChannelId::new(TICKET_PANEL_CHANNEL));
    let msgs = channel_id.messages(http, GetMessages::new().limit(20)).await?;
    if let Some(m) = msgs.iter().find(|m| is_panel(m)) {
        let edit = EditMessage::new().embed(panel_embed()).components(vec![panel_select_row()]);
        if Banner::Ticket.refresh(http, m, edit).await? {
            info!(channel = %channel_id, msg = %m.id, "ticket panel updated in place");
        } else {
            info!(channel = %channel_id, msg = %m.id, "ticket panel already up to date");
        }
        return Ok(());
    }
    let msg = CreateMessage::new()
        .add_file(Banner::Ticket.attachment())
        .embed(panel_embed())
        .components(vec![panel_select_row()]);
    channel_id.send_message(http, msg).await?;
    info!(channel = %channel_id, "ticket panel created on boot");
    Ok(())
}

async fn resolve_panel_channel(http: &Http, guild_id: GuildId) -> Option<ChannelId> {
    let channels = guild_id.channels(http).await.ok()?;
    channels.iter().find(|(_, c)| c.name.contains("open-ticket")).map(|(id, _)| *id)
}
