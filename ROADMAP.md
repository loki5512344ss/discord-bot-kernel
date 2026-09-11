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
```

Роли: `@Verified` (кнопка в #welcome), `@Blogger`, `@Support`, `@Staff`, `@Muted`.

---

## Phase 1 - Ядро ✅ (скелет готов)
- [x] Rust 1.82, tokio, serenity 0.12 + poise 0.6 (slash + prefix `!`)
- [x] `config/loader` dotenv → BotConfig, `db/pool` WAL + FK=ON (фикс C6 старого бота)
- [x] `db/migration` idempotent транзакции (фикс C5)
- [x] `core/lifecycle` trait, `core/bot` ReadyEvent + graceful shutdown
- [x] `feature/help|info|admin` - `!ping`, `!serverinfo`, `!shutdown` (owner)
- [x] `ds-setup` CLI: `inspect` / `bootstrap --dry-run/--apply` (BOT token, user-token с ворнингом)

**Запуск:**
```bash
cp .env.example .env  # вставь DISCORD_BOT_TOKEN, GUILD_ID
cargo run --bin bot
cargo run --bin ds-setup -- inspect --guild-id 123
cargo run --bin ds-setup -- bootstrap --guild-id 123 --dry-run
```

## Phase 2 - Верификация и доступ (guild 1509503154708811837, next)
- [ ] **Verify (embed+✅)** - `#✅｜verify` read-only (`@everyone: VIEW ✅, SEND ❌, ADD_REACTIONS ✅`), embed + `✅` реакция → выдача `✅ Verified` (1509503647480811610), бот роль выше Verified, `GUILD_MESSAGE_REACTIONS` + partials. Кнопка - альтернатива, но делаем реакцию как просил. Фикс F3: проверка `MANAGE_ROLES`.
- [ ] `!mute/!unmute/!warn/!purge/!slowmode` - атомарные транзакции, без `UndeclaredThrowableException` (фикс C1)
- [ ] Logging в #mod-logs (join/leave, edit/delete, mod actions)
- [ ] RateLimiter + `core/queue` Semaphore 100 (уже заготовка)

## Phase 3 - Блог-фичи (главное отличие от старого бота)
- [ ] RSS/Telegram → #blog-feed (webhook, `reqwest` + `feed-rs`)
- [ ] `!post` - анонс поста (embed с cover, линк)
- [x] GitHub webhook → #changelog (нативная интеграция Discord, без кода: GitHub webhook URL = `<discord-webhook-url>/github`)
- [x] Ticket (кнопки в #open-ticket → приватный канал/тред в SUPPORT, overwrites за 1 операцию, claim, blacklist, transcript → hastebin) - `feature/ticket`

## Phase 4 - Комьюнити
- [x] Welcome embed (`feature/welcome`)
- [x] Counters (member count в voice-канале, `feature/counter`, refresh 10 мин)
- [x] Polls (`/poll вопрос; вариант1; вариант2`, embed + реакции цифрами, `feature/poll`)
- [x] Scheduler / напоминания (`/remind 10m текст`, личные, SQLite + фоновый воркер 30с, `feature/remind`)
- [x] Temp voice (`#create-voice` → временный канал, `feature/voice`)

## Phase 5 - Опционально (AI)
- [ ] AI FAQ (DeepSeek/GPT) в #faq - только по базе знаний блога
- [ ] Auto-mod (спам/ссылки)

---

## Уроки старого бота - что НЕ повторяем
F1,F2,F4 - экономика: атомарные `UPDATE ... WHERE cash>=?`, daily `WHERE last_daily != today`.
F8/C1 - не блокировать gateway: все DB/HTTP вне EventHandler, `tokio::spawn`, `sqlx` async.
C6 - `foreign_keys=ON` через `SqliteConnectOptions::foreign_keys(true)`, не одним `PRAGMA`.
M1 - `destroy()` на `ScheduledExecutorService` → в Rust `JoinHandle::abort` / `pool.close().await`.

## Качество
`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo audit`. Лимит 200 строк - `rustfmt max_width=100`.
Каждый `feature/*` ≤4 файла, >150 строк → дробить (`service/`, `repository/`).
