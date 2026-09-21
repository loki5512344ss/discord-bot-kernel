//! Help categories - static mapping of category → emoji, color, description.
//! Mirrors Java `HelpCategories.java` but trimmed for Rust kernel (no economy/music).

use serenity::all::Colour;

/// Category id is the display name (sorted alphabetically in selector).
pub struct Cat {
    pub emoji: &'static str,
    pub color: Colour,
    pub desc: &'static str,
}

pub fn all() -> Vec<(&'static str, Cat)> {
    vec![
        (
            "Information",
            Cat {
                emoji: "ℹ️", color: Colour::from_rgb(52, 152, 219), desc: "Server & bot info"
            },
        ),
        (
            "Moderation",
            Cat {
                emoji: "🛡️",
                color: Colour::from_rgb(231, 76, 60),
                desc: "Mod tools (requires perms)",
            },
        ),
        (
            "Tickets",
            Cat {
                emoji: "🎫",
                color: Colour::from_rgb(88, 101, 242),
                desc: "Support tickets & blacklist",
            },
        ),
        (
            "Utility",
            Cat {
                emoji: "🔧",
                color: Colour::from_rgb(46, 204, 113),
                desc: "Polls, reminders, blog",
            },
        ),
    ]
}

pub fn get(cat: &str) -> Option<Cat> {
    all().into_iter().find(|(n, _)| *n == cat).map(|(_, c)| c)
}

/// Maps poise command name → category name.
pub fn category_of(cmd_name: &str) -> &'static str {
    match cmd_name {
        "help" | "ping" | "serverinfo" => "Information",
        "verify_setup" | "clear" | "slowmode" | "warn" | "shutdown" => "Moderation",
        "ticket_panel" | "ticket_blacklist_add" | "ticket_blacklist_remove" => "Tickets",
        "poll" | "remind" | "blog_post" => "Utility",
        _ => "Utility",
    }
}
