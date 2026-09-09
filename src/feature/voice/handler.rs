//! Temp voice handler for 1509503694088179862 (➕｜create-voice).
//! Best practice (VoiceMaster/TempVoice): hub channel -> clone temp, auto-delete when empty.
//! No `sleep` without `// SAFETY:`.

use serenity::all::{ChannelId, ChannelType, Context, CreateChannel, VoiceState};
use tracing::{info, warn};

const HUB: ChannelId = ChannelId::new(1509503694088179862);
const VOICE_CATEGORY: u64 = 1509503649062326484; // VOICE

/// Called on `VoiceStateUpdate`. Creates temp channel on hub join, deletes when empty.
pub async fn on_voice_state_update(ctx: Context, old: Option<VoiceState>, new: VoiceState) {
    // Join hub -> create temp
    if let Some(channel_id) = new.channel_id {
        if channel_id == HUB {
            if let Some(guild_id) = new.guild_id {
                if let Some(member) = new.member.clone() {
                    let name = format!("🔊 {}", member.display_name());
                    match guild_id
                        .create_channel(
                            &ctx.http,
                            CreateChannel::new(&name)
                                .kind(ChannelType::Voice)
                                .category(ChannelId::new(VOICE_CATEGORY)),
                        )
                        .await
                    {
                        Ok(ch) => {
                            let _ = guild_id.move_member(&ctx.http, new.user_id, ch.id).await;
                            info!(user = %new.user_id, channel = %ch.id, "temp voice created");
                        }
                        Err(e) => warn!(error = %e, "failed to create temp voice"),
                    }
                }
            }
        }
    }
    // Leave -> delete empty temp channels (not hub, not afk)
    if let Some(old_state) = old {
        if let Some(old_channel) = old_state.channel_id {
            if old_channel != HUB && old_channel.get() != 1509503696566882324 {
                if let Ok(serenity::all::Channel::Guild(g)) =
                    old_channel.to_channel(&ctx.http).await
                {
                    if let Ok(members) = g.members(&ctx.cache) {
                        if members.is_empty() {
                            let _ = g.delete(&ctx.http).await;
                            info!(channel = %old_channel, "temp voice deleted (empty)");
                        }
                    }
                }
            }
        }
    }
}
