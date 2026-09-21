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

pub async fn on_message_edit(ctx: &Context, event: &MessageUpdateEvent) {
    let Some(content) = event.content.clone() else { return };
    if content.len() > 1000 {
        return;
    }
    let embed = CreateEmbed::new()
        .title("Message edited")
        .description(format!(
            "<#{}> by <@{}>\n```{}```",
            event.channel_id,
            event.author.as_ref().map(|a| a.id.to_string()).unwrap_or_default(),
            content.chars().take(900).collect::<String>()
        ))
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
