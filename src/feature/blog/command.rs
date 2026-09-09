//! Manual slash command `/blog_post` for testing backend integration.
//! Admin only (`MANAGE_GUILD`), uses `defer` to avoid gateway delays.

use crate::feature::Ctx;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use tracing::info;

use super::service::{Bilingual, NewsPayload};

/// Converts string to bilingual (duplicates to ru/en).
fn bilingual(s: &str) -> Bilingual {
    Bilingual { ru: s.to_string(), en: s.to_string() }
}

/// `/blog_post` - manual blog creation for testing (admin only).
#[poise::command(slash_command, prefix_command, required_permissions = "MANAGE_GUILD", guild_only)]
pub async fn blog_post(
    ctx: Ctx<'_>,
    #[description = "Post title"] title: String,
    #[description = "Markdown content"] content: String,
) -> anyhow::Result<()> {
    ctx.defer().await?;
    let author_id = ctx.author().id.to_string();
    let channel_id = ctx.channel_id().to_string();
    // Use message id as unique per command invocation (timestamp based).
    let message_id = format!("cmd-{}-{}", author_id, chrono_timestamp());
    let tags = vec!["manual".to_string()];
    let excerpt = content.chars().take(200).collect::<String>();
    let payload = NewsPayload {
        title: bilingual(&title),
        content: bilingual(&content),
        excerpt: Some(bilingual(&excerpt)),
        cover_image: None,
        images: Vec::new(),
        tags,
        channel_id,
        message_id,
        author_id: author_id.clone(),
    };
    match super::service::post_news(payload).await {
        Ok(resp) => {
            info!(slug = %resp.slug, id = %resp.id, "manual blog_post success");
            let embed = CreateEmbed::new()
                .title("✅ Blog posted")
                .description(format!("Slug: `{}`\nID: `{}`", resp.slug, resp.id))
                .color(0x0057_F387);
            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Blog post failed")
                .description(format!("{e}"))
                .color(0x00ED_4245);
            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        }
    }
    Ok(())
}

/// Returns current unix millis as string for idempotency key.
fn chrono_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
