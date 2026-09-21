//! Help select handler - edits help message when user picks category.

use super::categories;
use serenity::all::{
    ComponentInteraction, ComponentInteractionDataKind, Context, CreateActionRow, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};

/// Builds the category selector row - reused in main page and category pages.
pub fn selector_row() -> CreateActionRow {
    let options: Vec<CreateSelectMenuOption> = categories::all()
        .iter()
        .map(|(name, cat)| {
            CreateSelectMenuOption::new(
                format!("{} {}", cat.emoji, name),
                format!("category:{name}"),
            )
            .description(cat.desc)
            .emoji(cat.emoji.parse::<serenity::all::ReactionType>().unwrap_or('📌'.into()))
        })
        .collect();
    let menu = CreateSelectMenu::new("help:category", CreateSelectMenuKind::String { options })
        .placeholder("📂 Choose category")
        .min_values(1)
        .max_values(1);
    CreateActionRow::SelectMenu(menu)
}

fn category_commands(cat: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    // (name, usage, desc) - keep in sync with `feature/mod.rs` registration
    let all = vec![
        ("help", "/help [command]", "Shows help (this menu)"),
        ("ping", "/ping", "Latency check"),
        ("serverinfo", "/serverinfo", "Server member & channel info"),
        ("verify_setup", "/verify_setup", "Re-post verify embed (Staff)"),
        ("clear", "/clear <n>", "Bulk delete messages (Mod)"),
        ("slowmode", "/slowmode <sec>", "Set slowmode (Mod)"),
        ("warn", "/warn @user <reason>", "Warn user (Mod)"),
        ("shutdown", "/shutdown", "Graceful shutdown (Owner)"),
        ("ticket_panel", "/ticket_panel", "Re-post ticket panel (Staff)"),
        ("ticket_blacklist_add", "/ticket_blacklist_add @user <reason>", "Block user from tickets"),
        ("ticket_blacklist_remove", "/ticket_blacklist_remove @user", "Unblock ticket user"),
        ("poll", "/poll <q; opt1; opt2>", "Create poll with reactions"),
        ("remind", "/remind <10m> <text>", "Personal reminder"),
        ("blog_post", "/blog_post <url>", "Announce blog post"),
    ];
    all.into_iter().filter(|(n, _, _)| categories::category_of(n) == cat).collect()
}

fn build_category_embed(cat: &str, requester: &str) -> CreateEmbed {
    let c = categories::get(cat).unwrap_or(categories::Cat {
        emoji: "📌",
        color: serenity::all::Colour::from_rgb(88, 101, 242),
        desc: "",
    });
    let cmds = category_commands(cat);
    let mut desc = format!("**{} {}** — {}\n\n", c.emoji, cat, c.desc);
    if cmds.is_empty() {
        desc.push_str("_No commands in this category_");
    } else {
        for (name, usage, d) in &cmds {
            desc.push_str(&format!("`{}` — {}\n", usage, d));
            // add alias for search
            let _ = name;
        }
    }
    desc.push_str("\n*Use the selector below to switch categories.*");
    CreateEmbed::new()
        .title(format!("{} {}", c.emoji, cat))
        .description(desc)
        .color(c.color)
        .footer(CreateEmbedFooter::new(format!(
            "Requested by {requester} • {} commands",
            cmds.len()
        )))
}

fn build_main_embed(
    prefix: &str,
    requester: &str,
    total: usize,
    guild_icon: Option<String>,
) -> CreateEmbed {
    let mut cat_text = String::new();
    for (name, cat) in categories::all() {
        let count = category_commands(name).len();
        cat_text.push_str(&format!("{} **{}** — {} ({} cmds)\n", cat.emoji, name, cat.desc, count));
    }
    let mut e = CreateEmbed::new()
        .title("📚 Loki Bot — Help")
        .description(format!(
            "Hi! Slash + prefix `{}` bot. **{}** commands total.\n\n**📂 Categories:**\n{}\nUse the selector below to view commands.",
            prefix, total, cat_text
        ))
        .color(0x0058_65F2)
        .footer(CreateEmbedFooter::new(format!("Requested by {requester} • /help <command> for details")));
    if let Some(url) = guild_icon {
        e = e.thumbnail(url);
    }
    e
}

/// Handles `help:category` select. Edits the original help message.
pub async fn on_select(ctx: Context, interaction: ComponentInteraction) {
    if interaction.data.custom_id != "help:category" {
        return;
    }
    let value = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => {
            values.first().cloned().unwrap_or_default()
        }
        _ => return,
    };
    let Some(cat) = value.strip_prefix("category:") else { return };
    let embed = build_category_embed(cat, &interaction.user.name);
    let row = selector_row();
    // Edit original message via UpdateMessage response
    let _ = interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                serenity::all::CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .components(vec![row]),
            ),
        )
        .await;
}

/// Helper for command.rs to build main page.
pub fn main_embed(prefix: &str, requester: &str, guild_icon: Option<String>) -> CreateEmbed {
    let total: usize = categories::all().iter().map(|(n, _)| category_commands(n).len()).sum();
    build_main_embed(prefix, requester, total, guild_icon)
}
