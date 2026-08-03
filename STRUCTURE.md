# Bot Kernel — Loki Dev Studio

## Архитектура

```
bot_kernel/
├── build.gradle                          # Java 21, Gradle 9.6.1, JDA 6.4.2
├── settings.gradle                       # rootProject.name = 'bot-kernel'
├── gradle/libs.versions.toml             # version catalog
├── Dockerfile / docker-compose.yml
├── STRUCTURE.md                          # этот файл
│
└── src/
    ├── main/java/kernel/loki/
    │   ├── bootstrap/                    # Точка входа
    │   │   ├── App.java                  # main(), Guice.createInjector()
    │   │   └── ReadyListener.java        # JDA ReadyEvent → логирование
    │   │
    │   ├── config/                       # Конфигурация
    │   │   ├── ConfigKey.java            # enum с ключами + дефолты
    │   │   ├── ConfigLoader.java         # dotenv → BotConfig
    │   │   └── BotConfig.java            # Immutable record-like config
    │   │
    │   ├── command/                      # Команды
    │   │   ├── Command.java              # interface { name(), description(), execute() }
    │   │   ├── CommandContext.java       # record (message, args, channel, member, guild, author, event)
    │   │   ├── CommandCategory.java      # enum (ADMIN, HELP, INFO, MODERATION, ...)
    │   │   └── CommandResult.java        # sealed interface (Success|Error|Cooldown|NotFound)
    │   │
    │   ├── core/                         # Ядро
    │   │   ├── Bot.java                  # record (id, jda, type)
    │   │   ├── BotCluster.java           # Map<String, Bot>, getByType()
    │   │   ├── BotInitializer.java       # DI → регистрация → JDA → BotCluster
    │   │   ├── EventDispatcher.java      # VirtualThread executor, dispatch message → command
    │   │   ├── JdaFactory.java           # JDABuilder factory
    │   │   ├── Lifecycle.java            # interface { start(), shutdown() }
    │   │   ├── LifecycleManager.java     # Multibinder<Lifecycle> → startAll/shutdownAll
    │   │   ├── ListenerAggregator.java   # Собирает все listener'ы для JDA
    │   │   ├── MessageListener.java      # ListenerAdapter → EventDispatcher.dispatch()
    │   │   ├── ShutdownManager.java      # Graceful shutdown
    │   │   │
    │   │   ├── bootstrap/
    │   │   │   └── BootstrapPhase.java   # enum фаз старта
    │   │   ├── registry/
    │   │   │   ├── CommandRegistry.java  # ConcurrentHashMap<String, Command>
    │   │   │   ├── CommandRegistrar.java # @FunctionalInterface
    │   │   │   └── Registrar.java        # Авто-регистрация через Multibinder
    │   │   ├── security/
    │   │   │   ├── CooldownManager.java  # Caffeine cache per-key
    │   │   │   ├── RateLimitResult.java  # record
    │   │   │   └── RateLimiter.java      # Sliding window rate limit
    │   │   └── exception/
    │   │       ├── CommandException.java
    │   │       └── CommandExceptionHandler.java
    │   │
    │   ├── database/                     # База данных
    │   │   ├── Database.java             # HikariCP + SQLite, Lifecycle
    │   │   ├── DatabaseModule.java       # Guice module
    │   │   ├── DatabaseSchema.java       # Константы колонок
    │   │   ├── connection/
    │   │   │   ├── ConnectionPool.java   # interface
    │   │   │   ├── SqliteConnectionPool.java  # HikariDataSource
    │   │   │   └── ConnectionFactory.java    # Functional wrapper
    │   │   ├── migration/
    │   │   │   ├── Migration.java        # interface { version(), migrate() }
    │   │   │   ├── MigrationManager.java # Auto-discover + version tracking
    │   │   │   └── migrator/
    │   │   │       └── V1__InitialSchema.java  # guild_config, user_data, metrics
    │   │   └── repository/
    │   │       ├── Repository.java       # interface (findById, save, delete)
    │   │       └── BaseRepository.java   # Abstract with connection helpers
    │   │
    │   ├── di/
    │   │   └── CoreModule.java           # Главный Guice модуль
    │   │
    │   ├── embed/
    │   │   ├── EmbedColor.java           # Константы цветов
    │   │   ├── EmbedFactory.java         # EmbedBuilder factory
    │   │   └── EmbedTemplates.java       # Готовые шаблоны
    │   │
    │   ├── feature/                      # Фичи (подключаемые модули)
    │   │   ├── help/                     # !help
    │   │   ├── info/                     # !ping, !avatar, !serverinfo
    │   │   └── admin/                    # !shutdown
    │   │
    │   └── util/
    │       ├── LoggerDecorator.java      # SLF4J с маркерами
    │       ├── MessageUtil.java          # sanitize, truncate, deleteAfter
    │       └── TimeUtil.java             # форматирование, парсинг duration
    │
    ├── main/resources/
    │   ├── logback.xml
    │   └── assets/
    │
    └── test/java/kernel/loki/
        ├── command/CommandRegistryTest.java
        ├── config/BotConfigTest.java
        ├── core/security/RateLimiterTest.java
        └── embed/EmbedFactoryTest.java
```

## Принципы (KISS + DRY + SOLID)

| Принцип | Как соблюдается |
|---------|----------------|
| **S**ingle Responsibility | Каждый класс — одна сущность: Command → execute, Repository → data, Service → logic |
| **O**pen/Closed | Feature-модули добавляются без изменения ядра (Multibinder) |
| **L**iskov Substitution | Command → все команды interchangeable |
| **I**nterface Segregation | Repository<T,ID>, Migration, Lifecycle — минимальные интерфейсы |
| **D**ependency Inversion | Все через Guice DI, нет new() в бизнес-логике |
| **KISS** | record, enum, sealed interface — минимум boilerplate |
| **DRY** | BaseRepository, EmbedFactory, LifecycleManager — общие паттерны вынесены |

## Правила разработки

1. **Пакеты** — `kernel.loki.{слой}.{подслой}`
2. **Фичи** — каждая фича = папка в `feature/` с модулем, командами, сервисами, репозиториями
3. **Добавление фичи**:
   - Создать `*Module.java` extends `AbstractModule`
   - Забиндить `CommandRegistrar` через `Multibinder`
   - Забиндить `Lifecycle` если нужно
   - Подключить модуль в `CoreModule.configure()`
4. **Команды** — implements `Command`, регистрируются через `CommandRegistrar`
5. **БД** — миграции через `Migration`, репозитории extends `BaseRepository`
6. **Тесты** — JUnit 5, Mockito, testcontainers для БД
7. **Стиль** — Google Java Format, spotless, checkstyle, spotbugs

---

## План фич для Loki Dev Studio

### Сервер: Loki Dev (ID: 1509503154708811837)

```
📁 INFO
  #rules, #welcome, #announcements, #releases, #github-log
📁 COMMUNITY
  #general, #ru-chat, #en-chat, #ideas
📁 SUPPORT
  #open-ticket, #bug-report, #ticket-faq
📁 VOICE
  work-room-1, work-room-2, create-voice, afk
📁 STAFF (🔒 staff-only)
  #staff-chat, #mod-logs, #bot-commands, #ticket-logs
# ✅ verify (uncategorized)
```

### Очередность разработки

#### Phase 1 — Ядро ✅ (Готово)
- [x] Bootstrap, DI, Config
- [x] Command framework
- [x] Event dispatching (virtual threads)
- [x] Rate limiter + Cooldown
- [x] Database (HikariCP + SQLite + Migrations)
- [x] Base embed templates
- [x] Базовые команды: help, ping, avatar, serverinfo, shutdown

#### Phase 2 — Модерация и управление
- [ ] **Verification** — кнопка в `#verify`, выдача роли Verified + RU/EN
- [ ] **Auto-role** — выбор языка (RU/EN) через кнопки
- [ ] **Moderation** — mute, warn, kick, ban, purge, case-lookup
- [ ] **Logging** — audit log в `#mod-logs` (join/leave, message edit/delete, voice, mod actions)
- [ ] **Mute role** — роль `🔕 Muted`, синхронизация по гильдиям

#### Phase 3 — Поддержка клиентов
- [ ] **Ticket System** — кнопка в `#open-ticket`, создание канала в категории
- [ ] **Ticket panels** — FAQ в `#ticket-faq`
- [ ] **Bug Reports** — модальное окно в `#bug-report`
- [ ] **Client management** — БД клиентов, проектов, контрактов
- [ ] **Invoice/Payment tracking** — статусы оплат

#### Phase 4 — Комьюнити и продуктивность
- [ ] **Welcome** — приветствие новых участников в `#welcome`
- [ ] **Polls** — создание опросов в `#ideas`
- [ ] **GitHub integration** — вебхуки в `#github-log`
- [ ] **Scheduler** — отложенные сообщения, напоминания
- [ ] **Counters** — счётчики участников в названиях каналов

#### Phase 5 — Голосовые каналы
- [ ] **Temp Voice** — `#create-voice` → создание временного канала
- [ ] **Voice roles** — выдача роли за нахождение в войсе
- [ ] **Music** — Lavalink плеер

#### Phase 6 — AI и автоматизация
- [ ] **AI Assistant** — DeepSeek/GPT для ответов на вопросы клиентов
- [ ] **Auto-mod** — фильтр спама, ссылок, капслока
- [ ] **Stats** — статистика сервера, активность участников

#### Phase 7 — Профили и экономика (опционально)
- [ ] **Level/XP** — за сообщения и войс
- [ ] **Reputation** — +/rep
- [ ] **Profile cards** — ранговые карточки

---

## Стек технологий

- **Java 21** — virtual threads, pattern matching, records, sealed classes
- **Gradle 9.6.1** — version catalog, build cache
- **JDA 6.4.2** — Discord API
- **Guice 7.0** — dependency injection
- **HikariCP + SQLite** — database (может быть заменена на PostgreSQL)
- **SLF4J + Logback** — logging
- **Caffeine** — кэширование (cooldowns, rate limits)
- **JUnit 5 + Mockito** — тестирование
- **Checkstyle + SpotBugs + Spotless** — code quality
