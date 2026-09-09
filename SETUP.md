# SETUP - как запустить и настроить сервер блога

## 1. Токен (важно про user-token)
**Не используй user-токен для автоматизации - это self-bot, бан по ToS Discord.** 
Создай **Bot** в https://discord.com/developers/applications → Bot → Token, пригласи с правами `bot` + `applications.commands` + `MANAGE_GUILD` `MANAGE_CHANNELS` `MANAGE_ROLES` `SEND_MESSAGES`.

Если хочешь просканировать свой сервер без бота - `ds-setup` умеет `--user-token`, но выведет ворнинг и может улететь в бан.

## 2. Запуск ядра
```bash
cp .env.example .env
# вставь DISCORD_BOT_TOKEN, GUILD_ID (ПКМ по серверу → Copy ID), SUPER_ADMIN_ID
cargo run --bin bot
```

## 3. Инспекция сервера
```bash
cargo run --bin ds-setup -- inspect --guild-id 144603123456789
# или через .env: GUILD_ID=... cargo run --bin ds-setup -- inspect
```
Выведет каналы (`type` 0=text, 4=category, 2=voice), роли, цвета. Скопируй вывод - по нему правим структуру.

## 4. Bootstrap структуры блога
```bash
cargo run --bin ds-setup -- bootstrap --guild-id 144603123456789 --dry-run
cargo run --bin ds-setup -- bootstrap --guild-id 144603123456789 --apply
```
Dry-run только планирует `INFO/COMMUNITY/SUPPORT` (см. ROADMAP). Apply пока idempotent diff (создает недостающие). Требует BOT token.

## 5. Что дальше
- В `staroe/STRUCTURE.md` лежит старый план Loki Dev Studio - не копируй его 1в1, это другой бот.
- Добавляй фичи в `src/feature/<domain>/` - 1 папка = 1 модуль, `mod.rs` + `command.rs` + `service.rs` максимум.

## 6. Качество
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
