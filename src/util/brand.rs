//! Brand palette and embedded banner images (orange accent, matches loki-code.dev).

use serenity::all::{CreateAttachment, EditAttachments, EditMessage, Http, Message};

/// The bot's own user id (author check for boot-managed messages).
pub const BOT_USER_ID: u64 = 1527966600173457408;

/// Primary accent (the site's orange).
pub const ACCENT: u32 = 0x00FF_793F;
/// Lighter accent for secondary surfaces.
#[allow(dead_code)]
pub const ACCENT_SOFT: u32 = 0x00FF_B27A;
/// Positive outcomes (warm gold keeps the palette orange-leaning).
pub const OK: u32 = 0x00FF_CC38;
/// Errors and destructive actions.
pub const DANGER: u32 = 0x00FF_4D36;
/// Warnings.
pub const WARN: u32 = 0x00FF_A94D;

/// Bundled banner images, shown at the top of embeds.
#[derive(Clone, Copy)]
pub enum Banner {
    Ticket,
    Verify,
    Welcome,
    Help,
    /// Vertical 3:4 posters for `/help` category pages.
    HelpInformation,
    HelpModeration,
    HelpTickets,
    HelpUtility,
}

impl Banner {
    /// Poster for a `/help` category (by display name, e.g. `"Moderation"`).
    pub fn for_category(name: &str) -> Option<Self> {
        match name {
            "Information" => Some(Self::HelpInformation),
            "Moderation" => Some(Self::HelpModeration),
            "Tickets" => Some(Self::HelpTickets),
            "Utility" => Some(Self::HelpUtility),
            _ => None,
        }
    }

    fn file(self) -> &'static str {
        match self {
            Self::Ticket => "ticket.png",
            Self::Verify => "verify.png",
            Self::Welcome => "welcome.png",
            Self::Help => "help.png",
            Self::HelpInformation => "help-information.png",
            Self::HelpModeration => "help-moderation.png",
            Self::HelpTickets => "help-tickets.png",
            Self::HelpUtility => "help-utility.png",
        }
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Ticket => include_bytes!("../../assets/brand/ticket.png"),
            Self::Verify => include_bytes!("../../assets/brand/verify.png"),
            Self::Welcome => include_bytes!("../../assets/brand/welcome.png"),
            Self::Help => include_bytes!("../../assets/brand/help.png"),
            Self::HelpInformation => include_bytes!("../../assets/brand/help-information.png"),
            Self::HelpModeration => include_bytes!("../../assets/brand/help-moderation.png"),
            Self::HelpTickets => include_bytes!("../../assets/brand/help-tickets.png"),
            Self::HelpUtility => include_bytes!("../../assets/brand/help-utility.png"),
        }
    }

    /// Attachment to add to the message that carries the embed.
    pub fn attachment(self) -> CreateAttachment {
        CreateAttachment::bytes(self.bytes(), self.file())
    }

    /// `attachment://` URL for `CreateEmbed::image`.
    pub fn url(self) -> String {
        format!("attachment://{}", self.file())
    }

    /// True when `msg` already carries this banner inside an accent-coloured embed.
    pub fn is_on(self, msg: &Message) -> bool {
        msg.attachments.iter().any(|a| a.filename == self.file())
            && msg.embeds.iter().any(|e| e.image.is_some() && e.colour.map(|c| c.0) == Some(ACCENT))
    }

    /// Re-renders an existing message in the current design, swapping its attachments for
    /// this banner. Returns `false` (no API call) when the message is already up to date.
    pub async fn refresh(
        self,
        http: &Http,
        msg: &Message,
        edit: EditMessage,
    ) -> anyhow::Result<bool> {
        if self.is_on(msg) {
            return Ok(false);
        }
        let edit = edit.attachments(EditAttachments::new().add(self.attachment()));
        msg.channel_id.edit_message(http, msg.id, edit).await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::Banner;

    #[test]
    fn every_help_category_has_a_poster() {
        for (name, _) in crate::feature::help::categories::all() {
            let b = Banner::for_category(name).expect("poster for category");
            assert!(b.url().starts_with("attachment://help-"));
            assert!(!b.bytes().is_empty());
        }
    }
}
