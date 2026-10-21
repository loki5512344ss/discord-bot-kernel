//! Ticket handler - mod (topics, panel, routing). Re-exports submodules.

pub mod claim;
pub mod create;
pub mod interaction;

pub use crate::feature::ticket::service::panel::ensure_ticket_panel;
pub use interaction::on_select;

use serenity::all::{
    Context, CreateActionRow, CreateEmbed, CreateEmbedFooter, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption,
};

pub const SUPPORT_CATEGORY: u64 = 1547213779224363008;
pub const TICKET_LOGS: u64 = 1509503717127356446;

/// Canonical topics - single source of truth.
pub struct Topic {
    pub value: &'static str,
    pub label: &'static str,
    pub emoji: char,
    pub desc: &'static str,
}
pub const TOPICS: &[Topic] = &[
    Topic {
        value: "general",
        label: "General support",
        emoji: '🛠',
        desc: "Questions about bot / server",
    },
    Topic { value: "bug", label: "Bug report", emoji: '🐛', desc: "Report a bug or error" },
    Topic {
        value: "appeal", label: "Mute / ban appeal", emoji: '🔇', desc: "Appeal a punishment"
    },
    Topic { value: "report", label: "Report user", emoji: '😡', desc: "Report rule violation" },
    Topic {
        value: "owners",
        label: "Contact owners",
        emoji: '👑',
        desc: "Private contact with owners",
    },
];

pub fn topic_label(value: &str) -> Option<&'static str> {
    TOPICS.iter().find(|t| t.value == value).map(|t| t.label)
}

pub fn panel_embed() -> CreateEmbed {
    CreateEmbed::new()
        .image(crate::util::brand::Banner::Ticket.url())
        .title("🎫 Support tickets")
        .description(
            "**Choose a topic below to open a private ticket.**\n\n\
            • Staff will respond shortly — please describe your issue clearly.\n\
            • One open ticket per user.\n\
            • Abuse / spam → blacklist.",
        )
        .color(crate::util::brand::ACCENT)
        .footer(CreateEmbedFooter::new("Loki Dev • Select topic → fill form → ticket created"))
}

pub fn panel_select_row() -> CreateActionRow {
    let options: Vec<CreateSelectMenuOption> = TOPICS
        .iter()
        .map(|t| CreateSelectMenuOption::new(t.label, t.value).description(t.desc).emoji(t.emoji))
        .collect();
    let menu = CreateSelectMenu::new("ticket_topic", CreateSelectMenuKind::String { options })
        .placeholder("Choose topic…")
        .min_values(1)
        .max_values(1);
    CreateActionRow::SelectMenu(menu)
}

/// Handles ticket buttons (claim/close) + legacy create buttons.
pub async fn on_button(ctx: Context, interaction: serenity::all::ComponentInteraction) {
    match interaction.data.custom_id.as_str() {
        "ticket_general" | "ticket_bug" => create::handle_create_legacy(ctx, interaction).await,
        "ticket_claim" => claim::handle_claim(ctx, interaction).await,
        "ticket_close" => claim::handle_close(ctx, interaction).await,
        "ticket_reopen" => claim::handle_reopen(ctx, interaction).await,
        "ticket_transcript" => claim::handle_transcript(ctx, interaction).await,
        "ticket_delete" => claim::handle_delete(ctx, interaction).await,
        _ => {}
    }
}

pub(crate) fn sanitize_name(s: &str) -> String {
    let lower = s.to_lowercase().replace(' ', "-");
    let mut out = String::new();
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let collapsed = out.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if collapsed.is_empty() { "user".into() } else { collapsed }
}

pub(crate) async fn ephemeral(
    ctx: &Context,
    i: &serenity::all::ComponentInteraction,
    content: String,
) {
    let _ = i
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                serenity::all::CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await;
}

pub(crate) fn allow_view(id: serenity::all::RoleId) -> serenity::all::PermissionOverwrite {
    serenity::all::PermissionOverwrite {
        allow: serenity::all::Permissions::VIEW_CHANNEL
            | serenity::all::Permissions::SEND_MESSAGES
            | serenity::all::Permissions::READ_MESSAGE_HISTORY,
        deny: serenity::all::Permissions::empty(),
        kind: serenity::all::PermissionOverwriteType::Role(id),
    }
}
pub(crate) fn build_ticket_overwrites(user_id: u64) -> Vec<serenity::all::PermissionOverwrite> {
    let everyone = serenity::all::RoleId::new(1509503154708811837);
    let staff = serenity::all::RoleId::new(1547203757711040572);
    let owner = serenity::all::RoleId::new(1547203762035236925);
    let moderator = serenity::all::RoleId::new(1547203753373859901);
    let user = serenity::all::UserId::new(user_id);
    vec![
        serenity::all::PermissionOverwrite {
            allow: serenity::all::Permissions::empty(),
            deny: serenity::all::Permissions::VIEW_CHANNEL,
            kind: serenity::all::PermissionOverwriteType::Role(everyone),
        },
        serenity::all::PermissionOverwrite {
            allow: serenity::all::Permissions::VIEW_CHANNEL
                | serenity::all::Permissions::SEND_MESSAGES
                | serenity::all::Permissions::READ_MESSAGE_HISTORY,
            deny: serenity::all::Permissions::empty(),
            kind: serenity::all::PermissionOverwriteType::Member(user),
        },
        allow_view(staff),
        allow_view(owner),
        allow_view(moderator),
    ]
}
