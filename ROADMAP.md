# ROADMAP - Loki Blog Bot (Rust kernel)

> Цель: не клон `discord-bot` (экономика/музыка/левел), а бот для **блога/портфолио** - чистый, KISS, ≤200 строк/файл, ≤4 файла/папку.

---

## Сервер блога - желаемая структура Discord

```
📁 INFO          #rules, #welcome, #announcements, #blog-feed (RSS → webhook), #changelog
📁 COMMUNITY     #general, #ideas, #showcase (юзеры постят проекты), #feedback
📁 SUPPORT       #open-ticket, #faq
📁 VOICE         create-voice (temp), afk
📁 STAFF (🔒)    #mod-logs, #bot-logs
   ↑ counter 👥 Members: N голосовой канал ВНЕ категории в самом верху (pos 0, bots excluded)
```

Роли: `@Verified` (кнопка в #welcome), `@Blogger`, `@Support`, `@Staff`, `@Muted`.

---

## Phase 1 - Ядро ✅ (скелет готов)
- [x] Rust 1.82, tokio, serenity 0.12 + poise 0.6 (slash + prefix `!`)
- [x] `config/loader` dotenv → BotConfig, `db/pool` WAL + FK=ON (фикс C6 старого бота)
- [x] `db/migration` idempotent транзакции (фикс C5)
- [x] `core/lifecycle` trait, `core/bot` ReadyEvent + graceful shutdown
- [x] `feature/help|info|admin` - `!ping`, `!serverinfo`, `!shutdown` (owner) → теперь help интерактивный (селект категорий)
- [x] `ds-setup` CLI: `inspect` / `bootstrap --dry-run/--apply` (BOT token, user-token с ворнингом)

**Запуск:**
```bash
cp .env.example .env  # вставь DISCORD_BOT_TOKEN, GUILD_ID
cargo run --bin bot
cargo run --bin ds-setup -- inspect --guild-id 123
cargo run --bin ds-setup -- bootstrap --guild-id 123 --dry-run
```

## Phase 2 - Верификация и доступ (guild 1509503154708811837)
- [x] **Verify (embed+✅)** - `#✅｜verify` read-only, auto-embed на старте, `✅` → `✅ Verified`
- [x] Help - переписан как в Java: `help/categories.rs` + `help/handler.rs` + селект `help:category` → `UpdateMessage`, цвета/эмодзи по категориям, фильтр мод-прав (Info/Mod/Tickets/Utility), как `HelpMenuBuilder.java`/`HelpMenuHandler.java`
- [x] Counter fix - `feature/counter/task.rs`: вне категории `position(0)`, каждый старт проверяет `parent_id` и `position`, поднимает вверх; считает только живых (пагинация `members(1000, after)`, `!user.bot`), fallback на `approximate_member_count`
- [ ] `!mute/!unmute/!warn/!purge/!slowmode` - атомарные транзакции
- [ ] Logging в #mod-logs (join/leave, edit/delete, mod actions)
- [ ] RateLimiter + `core/queue` Semaphore 100 (уже заготовка)

## Phase 3 - Блог-фичи (главное отличие от старого бота)
- [ ] RSS/Telegram → #blog-feed (webhook, `reqwest` + `feed-rs`)
- [ ] `!post` - анонс поста (embed с cover, линк)
- [x] GitHub webhook → #changelog (нативная интеграция Discord, без кода: GitHub webhook URL = `<discord-webhook-url>/github`)
- [x] Ticket v2 - SelectMenu `ticket_topic` + Modal `ticket_modal:<topic>` (subject/description) → приватный канал в SUPPORT, overwrites за 1 операцию, sanitize имени, claim, blacklist, transcript → hastebin - `feature/ticket/handler.rs` + auto-panel `ensure_ticket_panel()` на Ready (как verify)

### Phase 3.1 - Ticket hardening (TODO, приоритезировано 22.09.2026)
- [ ] **Add/Remove user** - `/ticket_add @user` `/ticket_remove @user` (staff): `MANAGE_CHANNELS` check, `channel.permission_overwrites` add/remove Member overwrite `VIEW+SEND`, лог в ticket-logs, DM добавленному.
- [ ] **Close reason modal + Reopen/Delete** - `Close` → modal `ticket_close_reason` (reason 10-500), затем `ControlPanel`: `Reopen` (снимает `archived`, восстанавливает overwrites), `Transcript` (haste + file), `Delete` (транскрипт → `ChannelId::delete`). Как `ticketPresentation.js:166` `buildClosedControlPanelRow()` (Reopen secondary, Save secondary, Delete danger).
- [ ] **Cooldown 5 мин + RateLimiter per user** - после `cleanup_ticket` писать в `ticket_cooldowns(user_id, guild_id, expires_at)`; при `on_select/on_modal` проверять `now < expires_at` → ephemeral `Wait 3m 12s`. Plus глобальный `RateLimiter` per-user 1 ticket/5m + burst check.
- [ ] **Orphan reconciliation on boot** - `ticketReconcile()` в `ready`: `SELECT tickets` → для каждого `guild.channels().fetch(ticket.channel_id)` если `None` → `DELETE FROM tickets/claims`, лог embed `Ticket Manually Deleted` (без транскрипта, как `threadDelete.js`), dm owner. Итого `ticketRepository.listOpenTickets()` аналог.
- [ ] **Presence `watching N tickets` + counter** - `Context::set_presence(ActivityData::watching(format!("{} tickets", count)))`, обновлять на `create/close/reconcile`; `ticket_count` из `SELECT COUNT(*) FROM tickets WHERE guild_id=?`.
- [ ] **Threads vs Channels** - сейчас `ChannelType::Text` в SUPPORT (лимит 500 каналов, видно в списке). При >50 тикетов/день → мигрировать на `ChannelType::PublicThread`/`PrivateThread` внутри `#open-ticket` (лимит 1000 активных тредов, авто-архив 24ч/7д). Флаг `TICKET_USE_THREADS=false` в env, абстрагировать `create_ticket_channel` vs `create_thread`.
- [ ] **Anti-spam** - в тикет-каналах: `slowmode 5s` по умолчанию, `mention` лимит (1 @everyone → warn), бот роль выше `Verified`/`Muted` (иначе `403 add_roles`), `MANAGE_ROLES`/`MANAGE_CHANNELS` проверке на старте.
- [ ] **Forum/channel tags vs одна категория** - см. § Forum Tags ниже.

## Phase 4 - Комьюнити
- [x] Welcome embed (`feature/welcome`)
- [x] Counters (member count в voice-канале, `feature/counter`, refresh 10 мин, теперь вне категории наверху)
- [x] Polls (`/poll вопрос; вариант1; вариант2`, embed + реакции цифрами, `feature/poll`)
- [x] Scheduler / напоминания (`/remind 10m текст`, личные, SQLite + фоновый воркер 30с, `feature/remind`)
- [x] Temp voice (`#create-voice` → временный канал, `feature/voice`)

## Phase 5 - Опционально (AI)
- [ ] AI FAQ (DeepSeek/GPT) в #faq - только по базе знаний блога
- [ ] Auto-mod (спам/ссылки)

---

## Forum / Channel Tags — вместо одной категории SUPPORT

**Что это:** Discord с 2022г поддерживает **Forum-каналы** (`ChannelType::Forum`). Внутри один канал-форум, а каждый тикет — пост-тред с тегами. Теги — это встроенные лейблы Discord (до 20 штук, цвет+эмодзи), например: `❓ General`, `🐛 Bug`, `🔇 Appeal`, `😡 Report`. Юзер при создании поста выбирает тег(и), фильтрация по тегам нативная.

**Как это соотносится с твоей текущей схемой:**
- Сейчас: `SUPPORT` — категория, внутри создаются `ticket-bug-username` каналы (`ChannelType::Text`, `parent_id=SUPPORT_CATEGORY`, `permissions` на каждого юзера). Минус: жрёт лимит 500 каналов, каждый тикет виден в сайдбаре всем стаффам сразу, надо руками чистить.
- Форум-альтернатива: создаёшь **один** канал `Forum` (`#support-tickets`, `ChannelType::Forum`, `available_tags=[ General, Bug, Appeal, Report, Owners ]` + `default_reaction_emoji`). Панель остаётся (`StringSelect` → modal), но `create_ticket` делает `forum.create_forum_post(title=subject, content=description, tags=[topicTag])` → пост-тред. Права наследуются от форума (private threads видны только `@everyone` deny + thread member). Преимущества и лимиты:

| Критерий | Каналы в категории | Forum tags |
|---|---|---|
| Лимит | 500 каналов/гильдия | 1000 активных тредов + неограниченно архивированных |
| Сайдбар | каждый тикет — отдельный канал (шум) | один форум-канал, внутри список постов (чисто) |
| Теги/фильтр | категория одна, топик в имени канала `ticket-bug-` | нативные теги (фильтр, поиск, цвет, required/optional) |
| Права | `PermissionOverwrite` на канал (кастом) | `thread_add_member` + `VIEW_CHANNEL` deny на форум |
| Транскрипт | `channel.messages().fetch()` | `thread.messages().fetch()` аналогично |
| Авто-архив | ручной `delete` | нативный `default_auto_archive_duration` 1ч/24ч/7д |
| Сортировка | вручную | по активности, по тегам |

**Рекомендация для Loki Dev:** пока <50 тикетов/день — оставляй **каналы** (проще дебажить, транскрипт/claim привычнее). При росте — мигрировать на **форум с тегами** без смены UX: селект/ модалка остаются, меняется только `service::create_ticket` (флаг `TICKET_USE_FORUM`). В форум-режиме `SUPPORT_CATEGORY` не нужен, теги создаются один раз на старте (`GET/PATCH /channels/{forum.id}` с `available_tags`). Это «канал-теги вместо категории» — один канал с 5 тегами заменяет 5 подканалов/префиксов.

---

## Уроки старого бота - что НЕ повторяем
F1,F2,F4 - экономика: атомарные `UPDATE ... WHERE cash>=?`, daily `WHERE last_daily != today`.
F8/C1 - не блокировать gateway: все DB/HTTP вне EventHandler, `tokio::spawn`, `sqlx` async.
C6 - `foreign_keys=ON` через `SqliteConnectOptions::foreign_keys(true)`, не одним `PRAGMA`.
M1 - `destroy()` на `ScheduledExecutorService` → в Rust `JoinHandle::abort` / `pool.close().await`.

## Качество
`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo audit`. Лимит 200 строк - `rustfmt max_width=100`.
Каждый `feature/*` ≤4 файла, >150 строк → дробить (`service/`, `repository/`).
