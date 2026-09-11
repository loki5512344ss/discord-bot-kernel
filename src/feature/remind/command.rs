//! `/remind` slash command - personal reminders, delivered in the invoking channel.

use crate::feature::Ctx;
use poise::CreateReply;

/// Parses `10m`, `2h`, `1d`, `30s` into seconds. No external crate - single suffix char.
fn parse_duration(input: &str) -> Option<u64> {
    let input = input.trim();
    let (digits, unit) = input.split_at(input.len().checked_sub(1)?);
    let value: u64 = digits.parse().ok()?;
    let secs = match unit {
        "s" => value,
        "m" => value.checked_mul(60)?,
        "h" => value.checked_mul(3600)?,
        "d" => value.checked_mul(86_400)?,
        _ => return None,
    };
    (secs > 0).then_some(secs)
}

/// `/remind <duration> <message>` - e.g. `/remind 10m stretch break`. Max 7 days.
#[poise::command(slash_command, guild_only)]
pub async fn remind(
    ctx: Ctx<'_>,
    #[description = "Duration, e.g. 30s, 10m, 2h, 1d"] duration: String,
    #[description = "Reminder text"] message: String,
) -> anyhow::Result<()> {
    let Some(seconds) = parse_duration(&duration) else {
        ctx.send(
            CreateReply::default()
                .content("Invalid duration. Use e.g. `30s`, `10m`, `2h`, `1d`.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };
    if seconds > 7 * 86_400 {
        ctx.send(CreateReply::default().content("Max duration is 7 days.").ephemeral(true)).await?;
        return Ok(());
    }
    super::service::insert(
        &ctx.data().pool,
        &ctx.author().id.to_string(),
        &ctx.channel_id().to_string(),
        &message,
        seconds,
    )
    .await?;
    ctx.send(
        CreateReply::default()
            .content(format!("⏰ Got it, I'll remind you in {duration}."))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
