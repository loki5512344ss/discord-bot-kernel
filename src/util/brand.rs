//! Brand palette and embedded banner images (orange accent, matches loki-code.dev).

use serenity::all::CreateAttachment;

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
