package kernel.loki.config;

import java.util.Map;

public class BotConfig {
    private final Map<String, String> values;

    public BotConfig(Map<String, String> values) {
        this.values = Map.copyOf(values);
    }

    public String get(ConfigKey key) {
        return values.get(key.key());
    }

    public String getOrDefault(ConfigKey key) {
        return values.getOrDefault(key.key(), key.defaultValue());
    }

    public int getInt(ConfigKey key) {
        return Integer.parseInt(getOrDefault(key));
    }

    public String getToken() { return get(ConfigKey.DISCORD_BOT_TOKEN); }
    public String getPrefix() { return getOrDefault(ConfigKey.BOT_PREFIX); }
    public String getActivity() { return getOrDefault(ConfigKey.BOT_ACTIVITY); }
    public String getSuperAdminId() { return get(ConfigKey.SUPER_ADMIN_ID); }
    public String getDbPath() { return getOrDefault(ConfigKey.DB_PATH); }
    public int getHealthPort() { return getInt(ConfigKey.HEALTH_HTTP_PORT); }
}
