//! Language picker on first join (poll/buttons). Requires GUILD_MEMBERS intent.
//! Flow: on GuildMemberAdd → send in #welcome or DM → buttons 🇷🇺 / 🇬🇧 → assign role → bot replies in that lang (util::i18n::from_roles).

use serenity::all::{Context, Member};

pub async fn on_member_join(_ctx: Context, _member: Member) {
    // let channel = welcome_channel_id (1509503664438378596)
    // channel.send_message(http, CreateMessage::new()
    //   .content(format!("<@{}> Welcome! Pick your language:", member.user.id))
    //   .components(vec![CreateActionRow::Buttons(vec![
    //     CreateButton::new("lang_ru").label("Русский").emoji('🇷🇺'),
    //     CreateButton::new("lang_en").label("English").emoji('🇬🇧'),
    //   ])])
    // ).await?;
}

pub async fn on_lang_button(_ctx: Context, _member: Member, _lang: &str) {
    // if custom_id == "lang_ru" → member.add_role(http, RU_ID).await?
    // if custom_id == "lang_en" → member.add_role(http, EN_ID).await?
    // followup ephemeral: t("lang.picked", lang)
}
