# Architecture - Rust kernel (edition 2024, guild 1509503154708811837)

```
bot-kernel/                          # cargo init --edition 2024 --name bot-kernel
 ├─ Cargo.toml                       # serenity 0.12.5 + poise 0.7.0 + sqlx 0.9.0 + tokio 1.53 + reqwest 0.13 (cargo add)
 ├─ .cargo/config.toml / rustfmt.toml / clippy.toml
 ├─ .env.example                     # BOT token only; USER token → tools/userbot/.env (isolated)
 ├─ ROADMAP.md / ARCHITECTURE.md / SETUP.md
 ├─ staroe/                          # архив Java kernel (21→25, build.gradle, src/kernel/loki)
 ├─ src/                             # Rust bot (BOT token, slash + prefix)
 │   ├─ main.rs                      # dotenv → config → pool (WAL/FK) → migration → poise → serenity
 │   ├─ config/                      # keys.rs, loader.rs (env→BotConfig)
 │   ├─ core/                        # lifecycle.rs, bot.rs (ReadyHandler + shard), dispatcher.rs, queue.rs, policy.rs
 │   ├─ db/                          # pool.rs (WAL, FK=ON, busy_timeout 5s), migration.rs (tx idempotent)
 │   ├─ feature/                     # ≤4 файла/папку, ≤200 строк/файл (см. SOLID)
 │   │   ├─ help/command.rs          # /help, !help (slash+prefix via poise)
 │   │   ├─ info/command.rs          # /ping, /serverinfo
 │   │   └─ admin/command.rs         # /shutdown (owners_only)
 │   ├─ util/logger.rs
 │   └─ bin/ds_setup.rs              # Rust inspector (BOT token) - дублирует JS, но не user-token
 └─ tools/userbot/                   # ← ОТДЕЛЬНАЯ ПАПКА для юзер-бота (JS, только просмотр)
     ├─ package.json (dotenv)        # node index.js inspect --user-token --guild-id 1509503154708811837
     ├─ index.js                     # fetch https://discord.com/api/v10/* с Authorization
     ├─ .env.example
     └─ README.md
```

> **Важно:** папка `tools/userbot` (была `tools/ds-inspector`) - **изолирована** от `src/`. Rust-бот никогда не импортирует user-token. JS-просмотрщик - read-only, не шлёт сообщения.

## Guild 1509503154708811837 - Loki Dev (проверено 09.09.2026, целевой - только он)
- Владелец: ты (loki_boba 1414975197269987471), 22.guilds, bot `kernel:1527966600173457408` уже в гильдии (permissions 18014398509481983). Не приглашать на другие - как просил.
- Каналы 27: INFO (rules, welcome, announcements, releases, github-log, ✅｜verify, roles), COMMUNITY (general, ru-chat, en-chat, ideas), SUPPORT (open-ticket, bug-report, ticket-faq), VOICE (work-room-1/2, create-voice, afk), STAFF (staff-chat, mod-logs, bot-commands, ticket-logs).
- Роли 16: dupe `👑 Loki/Staff/Client/Verified/RU/EN/Muted` (две копии - надо чистить), `kernel` (bot), `@everyone`.
- Токены валидны, хранятся в `.env` / `tools/userbot/.env` (gitignore), не коммитим.

## Code Style & Quality (English, Javadoc, no dead code, fmt/clippy)
- **Language:** all docs, comments, commit messages - **English only** (guild multi-language → EN default, RU via role `i18n.rs`). Chat in code = English.
- **Javadoc-style comments:** every `pub` item has `///` doc comment (like Java `/** ... */`): purpose, args, returns, example, `// SAFETY:` for sleeps. No undocumented `pub fn`.
- **Files:** `≤200` lines, `≤4` files per folder (`feature/<domain>/{mod,command,handler,service}.rs`). Split if >150.
- **Dead code forbidden:** `#[allow(dead_code)]` / `#[allow(unused)]` **forbidden** - delete unused code, don't hide it. CI fails on `dead_code`/`unused`. Use `cargo fix`, not `allow`.
- **Formatting mandatory:** `cargo fmt --check` must pass. `rustfmt.toml: edition 2024, max_width 100`. Run `cargo fmt` before commit.
- **Clippy mandatory:** `cargo clippy -- -D warnings` must pass. `clippy.toml: disallowed-methods = ["std::thread::sleep"]`. Fix all `clippy::pedantic`/`nursery` warnings, no `#[allow(clippy::...)]` without `// SAFETY:` + issue link.
- **CI:** `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo check` - blocked on failure.

## SOLID / KISS / DRY (≤200 строк/файл, ≤4 файла/папку)
- `Lifecycle` trait - ISP, каждая фича `impl Lifecycle`.
- `BotConfig` SRP (только env), `SqlitePool` DIP (инжект через `feature::Data`).
- `RateLimiter` + `JobQueue` DRY - не копипастим cooldown как в старом `D2` (7 команд копировали `getDeterministicValue`).
- `≤200` строк - `clippy::too_many_lines` + `rustfmt max_width 100`, `≤4` файла → `feature/<domain>/{mod,command,service,repository}`.

## Команды - slash + prefix (poise 0.7)
- Сейчас: `help`, `ping`, `serverinfo`, `shutdown`. Все `#[poise::command(slash_command, prefix_command)]`.
- План блога: `/verify` (кнопка → Verified), `/blog <url>` (RSS→embed), `/ticket` (кнопка), `/post` - все slash-first, prefix как fallback.
- Регистрация: `register_in_guild(GUILD_ID=1509503154708811837)` в dev (мгновенно), `register_globally` в проде (до 1ч). Не используй Global Sync при итерации (антипаттерн).

## Очередь / мультиядерность / анти-задержки (POLICY)
**Запрещено:** `tokio::time::sleep`, `std::thread::sleep`, `std::thread::park` внутри `feature/*/command.rs` **без** комментария `// SAFETY: <причина + длительность + jitter>`.

- **Почему:** блокировка gateway → `Missed heartbeat`, 429, CloudFlare ban. Старый бот падал на `HealthHttpServer single thread` (A07), `TrackScheduler` утечках, `M!play sleep 4с` (M2).
- **Как правильно:**
  1. `ctx.defer().await?` → ответ `Thinking...`, затем тяжёлая работа в `tokio::spawn` / `JobQueue`.
  2. `core/queue.rs` - bounded `tokio::sync::Semaphore(100)` + `mpsc::channel(256)` (аналог `MaxHandlerConcurrency=100` из go антипаттерна). Burst не спавнит unbounded tasks.
  3. `RateLimiter` per-user + global token bucket (429 `Retry-After` + jitter, не хардкод `sleep(1000)`).
  4. `sqlx` async, `reqwest` с `timeout 10s + context::withTimeout` - не `context.Background()` бесконечно.
  5. `poise::FrameworkError::CommandPanic` - ловим панику handler'а, не крашим весь процесс.
- **Проверка:** `clippy.toml` `disallowed-methods = ["std::thread::sleep"]`, CI `grep -R 'sleep' src/feature --include='*.rs' | grep -v 'SAFETY:' && exit 1`.

```
src/core/queue.rs  - BoundedJobQueue (Semaphore 100, channel 256, backpressure)
src/core/policy.rs - документация запрета задержек
src/core/dispatcher.rs - RateLimiter (check per user → Option<Duration>)
```

## Верификация и тикеты - как правильно (нагуглено 09.09.2026)

### Верификация (требование: embed + реакция ✅, чат заблокирован)
- **Канал `#✅｜verify` (1509503655487737916):** только чтения для всех. Overwrites: `@everyone` → `VIEW_CHANNEL ✅`, `READ_MESSAGE_HISTORY ✅`, `ADD_REACTIONS ✅`, `SEND_MESSAGES ❌`, `ADD_REACTIONS` (add new) ✅. Остальные каналы скрыты пока нет `✅ Verified`.
- **Embed:** бот `/verify-setup` (admin only) постит embed `Title: Проверка • Description: Нажми ✅ чтобы получить доступ` + `msg.react('✅')`. Храним `message_id` в `guild_config.verify_message_id`.
- **Событие:** `on_raw_reaction_add` (`GUILD_MESSAGE_REACTIONS` intent + `partials: MESSAGE/CHANNEL/REACTION`). Проверяем `payload.message_id == verify_message_id && emoji == ✅ && !bot`. Даём роль `✅ Verified (1509503250997448754 / 1509503647480811610)` через `member.add_roles`. Роль бота **выше** Verified в иерархии, иначе 403 (частая ошибка StackOverflow).
- **Безопасность:** `Verification Level` в Guild Settings → `Medium` (email +5min) или `High` (+10min) + `@everyone` без `SEND_MESSAGES` в остальных каналах. Альтернатива кнопке - `Button` (components V2) современнее, но ты просил реакцию - делаем реакцию (Discord не рекомендует для новых, но работает).
- **Ошибки:** бот должен иметь `MANAGE_ROLES`, роль выше Verified, partials включены, не `reaction.emoji.id` для unicode.

### Тикет-система (SUPPORT)
- **Панель:** embed в `#🎫｜open-ticket` с кнопками (`General`, `Bug`, `Support`) - современный способ (не реакции). Старый `🎫` реакция - легаси, кнопки лучше (нет лимита 20 reactions, не надо `ADD_REACTIONS`).
- **Создание:** `on_interaction_create` (button) → `guild.create_text_channel(name=ticket-username-001, parent=SUPPORT, overwrites={@everyone: deny VIEW, user: allow VIEW+SEND, Staff: allow})` **в одну операцию** `overwrites` (не `set_permissions` по очереди - медленно, антипаттерн SO).
- **Варианты:** `channel` (до 500 каналов, видно в списке) vs `thread` (до 1000, не в лимит, авто-архив). Для Loki Dev - `channel` ок, для хайлоада - `thread`.
- **Управление:** `/close`, `/add`, `/remove`, claim, transcript (fetch messages → Hastebin/file), лог в `#🧾｜ticket-logs` + DM юзеру. Чёрный список per-guild.
- **Best practices:** категории по типам, claim чтобы не дублировать ответы, транскрипты для сокращения FAQ.

## Анти-принципы (что НЕ делать) - нагуглено 09.09.2026
Собрано из `discord.go/anti-patterns`, `discords.ai` (wiki 2026), `Rank.top` (rate limits), `discord.js`, `discord.py` issues:

### 1. Токены и секреты
- **Коммитить токены/`.env`** → бан, скам. Фикс: `dotenvy`, `.gitignore` + `DISCORD_BOT_TOKEN` из env, `Reset Token` при утечке (у тебя оба токена уже утекли - регенерируй!).
- **Hardcode webhook URL** → спам.

### 2. Блокировка gateway / задержки
- **Blocking event handler** (sleep, тяжёлый CPU/DB/HTTP в handler) → dispatch pipeline стоп, `Missed 2 heartbeats`. Фикс: `defer + tokio::spawn` (см. policy выше).
- **`sleep` без объяснения** → запрещено (policy). Нужен jitter/backoff, не `sleep(200ms)` в цикле.
- **Sync Blocking Loops** (тяжёлые вычисления в async loop) → фриз всех юзеров. Фикс: `spawn_blocking`.
- **Not Setting MaxHandlerConcurrency** → unbounded goroutines/tasks при burst. Фикс: semaphore 100.

### 3. Rate limits (429, CloudFlare)
- **Игнор 429 / хардкод таймер** → каскадный фейл, IP-бан (10k invalid /10min → CloudFlare). Фикс: парсить `X-RateLimit-Remaining/Reset-After/Retry-After`, глобальная очередь, `exponential backoff + jitter` (full/decorrelated). Не `get 429 then retry` без queue.
- **Global REST ~50 req/s, Gateway 1 Identify/5s, 120 events/60s per WS** → шарить `max_concurrency`, sharding при росте.
- **Channel rename 2/10min, message edit 5/5s** → queue, не `msg.edit()` 10 раз подряд.

### 4. Slash vs prefix
- **Глобальный sync при разработке** → ждать 1ч. Фикс: `GuildCommandSync(GUILD_ID)` для Loki Dev.
- **Второй initial response после Defer** → Discord reject. Фикс: после `defer` только `followup`.
- **Reply "invalid command" на несуществующие** → спам при коллизии префиксов. Фикс: silent fail, только `invalid args`.

### 5. Интенты и кэш
- **Все интеты `Intents::all()`** → OOM, верификация. Фикс: `GUILD_MESSAGES|MESSAGE_CONTENT|GUILDS|GUILD_MEMBERS` только нужные (у тебя уже так).
- **Неиспользуемые привилегированные `MESSAGE_CONTENT`, `Presence`** → доп. верификация.

### 6. Прочее
- **Invite scraping** (бот джойнит все сервера по инвайтам) → бан.
- **Greedy prefixes (`!,$,.`)** → коллизии, используй уникальный (`!` ок, но можно `!loki`).
- **Overuse mentions / @everyone** → бот-лупы, мьют.
- **Unbounded listeners / cooldown=0** → спам тяжёлых DB. Фикс: `CooldownManager` + `RateLimiter`.
- **Игнор паник** → краш всего бота. Фикс: `poise::builtins::on_error` + `HandlerPanicError`.
- **Embeds в Components V2** → reject, используй text components.
- **Not handling Promise rejections / unhandled Result** → молчаливый краш.

## От старого бота - что уже пофикшено
- Нет `God Module`/`PrefixCommandRegistrar 22 param` → per-feature `Module + Registrar`.
- Нет блокирующего `JDBC` на gateway → `sqlx::SqlitePool` async.
- `WAL + FK=ON + busy_timeout 5s` через `SqliteConnectOptions` (фикс `C6`), миграции транзакционно (фикс `C5`), `UndeclaredThrowableException` анврап не нужен (Rust `anyhow`).
- `RateLimiter` + `JobQueue` вместо `CachedThreadPool` (A05) и `M2/M3 sleep 4s` на gateway.
