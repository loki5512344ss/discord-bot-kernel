package kernel.loki.config;

public enum ConfigKey {
    DISCORD_BOT_TOKEN("DISCORD_BOT_TOKEN"),
    BOT_PREFIX("BOT_PREFIX", "!"),
    BOT_ACTIVITY("BOT_ACTIVITY", "!help"),
    SUPER_ADMIN_ID("SUPER_ADMIN_ID"),
    DB_PATH("DB_PATH", "data/bot.db"),
    HEALTH_HTTP_PORT("HEALTH_HTTP_PORT", "8712");

    private final String key;
    private final String defaultValue;

    ConfigKey(String key, String defaultValue) {
        this.key = key;
        this.defaultValue = defaultValue;
    }

    ConfigKey(String key) {
        this(key, null);
    }

    public String key() { return key; }
    public String defaultValue() { return defaultValue; }
    public boolean hasDefault() { return defaultValue != null; }
}
