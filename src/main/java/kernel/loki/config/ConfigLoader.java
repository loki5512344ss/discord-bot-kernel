package kernel.loki.config;

import io.github.cdimascio.dotenv.Dotenv;
import java.util.HashMap;
import java.util.Map;

public class ConfigLoader {
    private ConfigLoader() {}

    public static BotConfig load() {
        Dotenv dotenv = Dotenv.configure()
            .ignoreIfMissing()
            .load();

        Map<String, String> values = new HashMap<>();

        for (ConfigKey key : ConfigKey.values()) {
            String value = dotenv.get(key.key());
            if (value != null) {
                values.put(key.key(), value);
            } else if (key.hasDefault()) {
                values.put(key.key(), key.defaultValue());
            }
        }

        return new BotConfig(values);
    }

    public static BotConfig load(String path) {
        Dotenv dotenv = Dotenv.configure()
            .directory(path)
            .ignoreIfMissing()
            .load();

        Map<String, String> values = new HashMap<>();

        for (ConfigKey key : ConfigKey.values()) {
            String value = dotenv.get(key.key());
            if (value != null) {
                values.put(key.key(), value);
            } else if (key.hasDefault()) {
                values.put(key.key(), key.defaultValue());
            }
        }

        return new BotConfig(values);
    }
}
