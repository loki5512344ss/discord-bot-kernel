//! Help select handler - edits help message when user picks category.

use super::categories;
use crate::util::brand;
use serenity::all::{
    ButtonStyle, ComponentInteraction, ComponentInteractionDataKind, Context, CreateActionRow,
    CreateButton, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
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
        color: serenity::all::Colour::new(brand::ACCENT),
        desc: "",
    });
    let cmds = category_commands(cat);
    let mut desc = format!("{}\n\n", c.desc);
    if cmds.is_empty() {
        desc.push_str("_No commands in this category_");
    } else {
        for (_, usage, d) in &cmds {
            desc.push_str(&format!("**`{usage}`**\n-# {d}\n\n"));
        }
    }
    let mut e = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(format!("LOKI DEV · {}", cat.to_uppercase())))
        .title(format!("{} {}", c.emoji, cat))
        .description(desc)
        .color(c.color)
        .footer(CreateEmbedFooter::new(format!(
            "{} commands · requested by {requester}",
            cmds.len()
        )));
    if let Some(poster) = brand::Banner::for_category(cat) {
        e = e.image(poster.url());
    }
    e
}

fn build_main_embed(
    prefix: &str,
    requester: &str,
    total: usize,
    guild_icon: Option<String>,
) -> CreateEmbed {
    let mut e = CreateEmbed::new()
        .author(CreateEmbedAuthor::new("LOKI DEV · COMMAND CENTER"))
        .title("What can I do for you?")
        .description(format!(
            "**{total}** commands in **{}** categories. Pick one in the menu below.\n\
            -# Slash commands and the `{prefix}` prefix both work. `/help <command>` shows details.",
            categories::all().len()
        ))
        .image(brand::Banner::Help.url())
        .color(brand::ACCENT)
        .footer(CreateEmbedFooter::new(format!("Requested by {requester}")));
    for (name, cat) in categories::all() {
        let count = category_commands(name).len();
        e = e.field(
            format!("{} {}", cat.emoji, name),
            format!("{}\n-# {count} commands", cat.desc),
            true,
        );
    }
    if let Some(url) = guild_icon.filter(|u| !u.is_empty()) {
        e = e.thumbnail(url);
    }
    e
}

/// Action rows for the help message: category selector + overview/website buttons.
pub fn components() -> Vec<CreateActionRow> {
    vec![
        selector_row(),
        CreateActionRow::Buttons(vec![
            CreateButton::new("help:home")
                .label("Overview")
                .emoji('🏠')
                .style(ButtonStyle::Secondary),
            CreateButton::new_link("https://loki-code.dev").label("Website"),
        ]),
    ]
}

/// Handles `help:category` select. Edits the original help message.
pub async fn on_select(ctx: Context, interaction: ComponentInteraction) {
    // UpdateMessage always replaces attachments, so every page re-uploads its own image.
    let (embed, image) = match interaction.data.custom_id.as_str() {
        "help:home" => (main_embed("!", &interaction.user.name, None), Some(brand::Banner::Help)),
        "help:category" => {
            let value = match &interaction.data.kind {
                ComponentInteractionDataKind::StringSelect { values } => {
                    values.first().cloned().unwrap_or_default()
                }
                _ => return,
            };
            let Some(cat) = value.strip_prefix("category:") else { return };
            (build_category_embed(cat, &interaction.user.name), brand::Banner::for_category(cat))
        }
        _ => return,
    };
    let mut msg = serenity::all::CreateInteractionResponseMessage::new()
        .embed(embed)
        .components(components());
    if let Some(image) = image {
        msg = msg.add_file(image.attachment());
    }
    let _ =
        interaction.create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(msg)).await;
}

/// Helper for command.rs to build main page.
pub fn main_embed(prefix: &str, requester: &str, guild_icon: Option<String>) -> CreateEmbed {
    let total: usize = categories::all().iter().map(|(n, _)| category_commands(n).len()).sum();
    build_main_embed(prefix, requester, total, guild_icon)
}
