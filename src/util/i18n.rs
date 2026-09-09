// i18n - English default, RU/EN via roles (🇷🇺 RU 1509503254956998706 / 🇬🇧 EN 1509503258757038150)
// Policy: all user-facing messages English; if user has RU role -> Russian, else English.
// On first join: poll/buttons to pick language (see feature/lang).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }
}

/// Detect lang from member roles (serenity Member). Pass role IDs.
/// Detects language from member roles. Checks RU roles first, else English.
pub fn from_roles(role_ids: &[u64]) -> Lang {
    const RU: u64 = 1509503254956998706;
    const RU2: u64 = 1509503651411001374; // dupe guild role copy
    if role_ids.iter().any(|id| *id == RU || *id == RU2) { Lang::Ru } else { Lang::En }
}

pub fn t(key: &str, lang: Lang) -> &'static str {
    match (key, lang) {
        ("help.title", Lang::Ru) => "📚 Loki Dev - помощь",
        ("help.title", Lang::En) => "📚 Loki Dev - Help",
        ("help.desc", Lang::Ru) => "Выбери категорию ниже. Пагинация - кнопки/селект.",
        ("help.desc", Lang::En) => "Pick a category below. Pagination via buttons/select.",
        ("help.field.info", Lang::Ru) => "📌 Информация",
        ("help.field.info", Lang::En) => "📌 Info",
        ("help.field.moderation", Lang::Ru) => "🛡️ Модерация",
        ("help.field.moderation", Lang::En) => "🛡️ Moderation",
        ("help.field.tickets", Lang::Ru) => "🎫 Тикеты",
        ("help.field.tickets", Lang::En) => "🎫 Tickets",
        ("help.field.verify", Lang::Ru) => "✅ Верификация",
        ("help.field.verify", Lang::En) => "✅ Verification",
        ("help.footer", Lang::Ru) => "Используй /help <команда> для деталей",
        ("help.footer", Lang::En) => "Use /help <command> for details",
        ("verify.title", Lang::Ru) => "Проверка",
        ("verify.title", Lang::En) => "Verification",
        ("verify.desc", Lang::Ru) => {
            "Нажми ✅ чтобы получить доступ к серверу.\n\nЧат в этом канале заблокирован - только реакция."
        }
        ("verify.desc", Lang::En) => {
            "Click ✅ to get access to the server.\n\nChat is locked here - reaction only."
        }
        ("ticket.title", Lang::Ru) => "Поддержка",
        ("ticket.title", Lang::En) => "Support",
        ("ticket.desc", Lang::Ru) => "Нажми кнопку чтобы открыть тикет",
        ("ticket.desc", Lang::En) => "Click a button to open a ticket",
        ("ping.title", _) => "Pong!",
        ("serverinfo.title", Lang::Ru) => "Информация о сервере",
        ("serverinfo.title", Lang::En) => "Server Info",
        _ => "-",
    }
}
