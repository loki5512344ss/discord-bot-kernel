//! Mute/unmute via timeout + Muted role (1547203722684137492).

use crate::feature::Ctx;
use poise::CreateReply;
use serenity::all::{CreateEmbed, Timestamp};

const MUTED_ROLE: u64 = 1547203722684137492;

fn parse_duration(s: &str) -> Option<std::time::Duration> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    let (num_str, unit) = s.split_at(s.len() - 1);
    let n: u64 = if unit.chars().next().unwrap().is_ascii_digit() {
        s.parse().ok()?
    } else {
        num_str.parse().ok()?
    };
    let secs = match unit {
        "s" if !s.chars().last().unwrap().is_ascii_digit() => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86400,
        _ if s.chars().last().unwrap().is_ascii_digit() => n * 60,
        _ => return None,
    };
    if secs == 0 || secs > 2419200 {
        return None;
    }
    Some(std::time::Duration::from_secs(secs))
}

/// `/mute @user 10m reason` - timeout + Muted role.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn mute(
    ctx: Ctx<'_>,
    #[description = "User to mute"] user: serenity::all::User,
    #[description = "Duration 10m/1h/1d or seconds"] duration: String,
    #[description = "Reason"] reason: Option<String>,
) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };
    let dur = parse_duration(&duration).unwrap_or(std::time::Duration::from_secs(600));
    ctx.defer_ephemeral().await?;
    let mut member = guild_id.member(ctx.http(), user.id).await?;
    let secs =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            + dur.as_secs();
    let until = Timestamp::from_unix_timestamp(secs as i64).unwrap_or_else(|_| Timestamp::now());
    let _ = member.disable_communication_until_datetime(ctx.http(), until).await;
    let _ = member.add_role(ctx.http(), serenity::all::RoleId::new(MUTED_ROLE)).await;
    let reason = reason.unwrap_or_else(|| "No reason".to_string());
    let embed = CreateEmbed::new()
        .title(format!("Muted {}", user.name))
        .description(format!("Duration: {}s\nReason: {}", dur.as_secs(), reason))
        .color(0x00ED_4245);
    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    log_mod(&ctx, &format!("Mute <@{}> {}s reason: {}", user.id, dur.as_secs(), reason)).await;
    Ok(())
}

/// `/unmute @user`
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn unmute(
    ctx: Ctx<'_>,
    #[description = "User to unmute"] user: serenity::all::User,
) -> anyhow::Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };
    ctx.defer_ephemeral().await?;
    let mut member = guild_id.member(ctx.http(), user.id).await?;
    let _ = member.enable_communication(ctx.http()).await;
    let _ = member.remove_role(ctx.http(), serenity::all::RoleId::new(MUTED_ROLE)).await;
    ctx.send(
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .description(format!("Unmuted <@{}>", user.id))
                    .color(0x0057_F387),
            )
            .ephemeral(true),
    )
    .await?;
    log_mod(&ctx, &format!("Unmute <@{}>", user.id)).await;
    Ok(())
}

async fn log_mod(ctx: &Ctx<'_>, text: &str) {
    let ch = serenity::all::ChannelId::new(1509503710219210872);
    let _ = ch
        .send_message(
            ctx.http(),
            serenity::all::CreateMessage::new()
                .embed(CreateEmbed::new().title("Mod log").description(text).color(0x00ED_4245)),
        )
        .await;
}
