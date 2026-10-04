//! Mod-logs: join/leave/edit/delete → #mod-logs (1509503710219210872).

use serenity::all::{ChannelId, Context, CreateEmbed, CreateMessage, Member, MessageUpdateEvent};

const MOD_LOGS: ChannelId = ChannelId::new(1509503710219210872);

pub async fn on_member_join(ctx: &Context, member: &Member) {
    let embed = CreateEmbed::new()
        .title("Member joined")
        .description(format!("<@{}> `{}`", member.user.id, member.user.name))
        .color(0x0057_F387)
        .thumbnail(member.user.avatar_url().unwrap_or_default());
    let _ = MOD_LOGS.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
}

pub async fn on_member_leave(
    ctx: &Context,
    user: &serenity::all::User,
    guild_id: Option<serenity::all::GuildId>,
) {
    let embed = CreateEmbed::new()
        .title("Member left")
        .description(format!("<@{}> `{}` guild={:?}", user.id, user.name, guild_id))
        .color(0x00ED_4245);
    let _ = MOD_LOGS.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
}

/// Guards against non-edits: Discord sends `MESSAGE_UPDATE` with
/// `edited_timestamp = null` ~1s after `MESSAGE_CREATE` for embed-only
/// messages. Logging that would post a new embed, which gets its own spurious
/// update → infinite self-sustaining loop in #mod-logs.
pub async fn on_message_edit(ctx: &Context, event: &MessageUpdateEvent) {
    if event.edited_timestamp.is_none() {
        return;
    }
    // Never log #mod-logs itself (feedback guard).
    if event.channel_id == MOD_LOGS {
        return;
    }
    // Author lives in the event payload: serenity message cache is off
    // (`max_messages = 0`), so the `new: Option<Message>` bot filter never fires.
    // Webhooks report `bot = true` too (GitHub integrations update themselves).
    let Some(author) = event.author.as_ref() else { return };
    if author.bot {
        return;
    }
    let Some(content) = event.content.clone() else { return };
    if content.len() > 1000 {
        return;
    }
    let body = if content.is_empty() {
        "*content cleared*".to_string()
    } else {
        format!("```{}```", content.chars().take(900).collect::<String>())
    };
    let embed = CreateEmbed::new()
        .title("Message edited")
        .description(format!("<#{}> by <@{}>\n{}", event.channel_id, author.id, body))
        .color(0x00FAA81A);
    let _ = MOD_LOGS.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
}

pub async fn on_message_delete(
    ctx: &Context,
    channel_id: ChannelId,
    message_id: serenity::all::MessageId,
    guild_id: Option<serenity::all::GuildId>,
) {
    let embed = CreateEmbed::new()
        .title("Message deleted")
        .description(format!(
            "<#{}> message `{}` guild={:?}",
            channel_id,
            message_id.get(),
            guild_id
        ))
        .color(0x0057_F387);
    let _ = MOD_LOGS.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
}

pub async fn on_message_delete_bulk(
    ctx: &Context,
    channel_id: ChannelId,
    ids: &[serenity::all::MessageId],
) {
    let embed = CreateEmbed::new()
        .title("Messages purged")
        .description(format!("<#{}> {} messages bulk deleted", channel_id, ids.len()))
        .color(0x00ED_4245);
    let _ = MOD_LOGS.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
}
