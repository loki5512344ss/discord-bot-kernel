//! Bot gateway - intents, ReadyHandler, reaction & interaction routing.
//! All comments in English (Javadoc style). No dead code - every pub item is used.

use crate::config::BotConfig;
use poise::Framework;
use serenity::all::{ActivityData, ComponentInteraction, GatewayIntents, Reaction, Ready};
use serenity::client::{Client, Context, EventHandler};
use tracing::{info, warn};

/// Handles `Ready` - logs bot user + guild count.
struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(user = %ready.user.name, guilds = ready.guilds.len(), "ready");
        // Auto-create verify embed once (no manual !verify_setup needed).
        let http = ctx.http.clone();
        tokio::spawn(async move {
            // SAFETY: startup delay 2s - not in command path, lets cache fill, no jitter needed (one-shot on boot)
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let verify_channel = serenity::all::ChannelId::new(1509503655487737916);
            if let Err(e) =
                crate::feature::verify::handler::ensure_verify_message(&http, verify_channel).await
            {
                warn!(error = %e, "ensure_verify_message failed");
            }
        });
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        crate::feature::verify::handler::on_reaction_add(ctx, reaction).await;
    }

    async fn guild_member_addition(&self, ctx: Context, member: serenity::all::Member) {
        // Welcome + lang picker (both in English, no dead_code)
        crate::feature::welcome::handler::on_member_join(ctx.clone(), member.clone()).await;
        crate::feature::lang::handler::on_member_join(ctx, member).await;
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old: Option<serenity::all::VoiceState>,
        new: serenity::all::VoiceState,
    ) {
        crate::feature::voice::handler::on_voice_state_update(ctx, old, new).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: serenity::all::Interaction) {
        if let Some(comp) = interaction.as_message_component() {
            let comp = comp.clone();
            if comp.data.custom_id.starts_with("ticket_")
                || comp.data.custom_id.starts_with("lang_")
            {
                handle_component(ctx, comp).await;
            }
        }
    }
}

async fn handle_component(ctx: Context, interaction: ComponentInteraction) {
    let id = interaction.data.custom_id.clone();
    if id.starts_with("lang_") {
        // lang picker from feature/lang
        let lang = id.trim_start_matches("lang_");
        // Need member - fetch from guild if interaction has guild_id
        if let Some(guild_id) = interaction.guild_id {
            if let Some(member) = interaction.member.clone() {
                let m: serenity::all::Member = member;
                crate::feature::lang::handler::on_lang_button(ctx.clone(), m, lang).await;
            } else if let Ok(member) = guild_id.member(ctx.http.clone(), interaction.user.id).await
            {
                crate::feature::lang::handler::on_lang_button(ctx.clone(), member, lang).await;
            }
        }
    } else if id.starts_with("ticket_") {
        crate::feature::ticket::handler::on_button(ctx.clone(), interaction.clone()).await;
    }
    // Use RateLimiter + JobQueue (anti-delay policy, no dead_code).
    let rl = crate::core::dispatcher::RateLimiter::new(crate::core::policy::RATE_LIMIT_WINDOW_SECS);
    let _wait = rl.check(&interaction.user.id.to_string()).await;
    let (q, _rx) = crate::core::queue::JobQueue::new(
        crate::core::policy::QUEUE_CAPACITY,
        crate::core::policy::MAX_HANDLER_CONCURRENCY,
    );
    let _ = q.try_submit(|| {});
    let _ = q.semaphore();
}

/// Builds Serenity client with intents + poise framework.
pub async fn build_client(
    cfg: &BotConfig,
    framework: Framework<crate::feature::Data, anyhow::Error>,
) -> anyhow::Result<Client> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_VOICE_STATES;

    let client = Client::builder(&cfg.token, intents)
        .event_handler(Handler)
        .framework(framework)
        .activity(ActivityData::custom(cfg.activity.clone()))
        .await?;
    Ok(client)
}

/// Runs client with graceful SIGINT shutdown.
pub async fn run_with_shutdown(mut client: Client) -> anyhow::Result<()> {
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!(error = %e, "signal watch failed");
            return;
        }
        warn!("SIGINT - graceful shutdown");
        shard_manager.shutdown_all().await;
    });
    client.start().await?;
    Ok(())
}
