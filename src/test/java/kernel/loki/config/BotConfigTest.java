package kernel.loki.config;

import static org.junit.jupiter.api.Assertions.*;
import java.util.Map;
import org.junit.jupiter.api.Test;

class BotConfigTest {
    @Test
    void shouldReturnDefaultForMissingOptionalKey() {
        BotConfig config = new BotConfig(Map.of());
        assertEquals("!", config.getOrDefault(ConfigKey.BOT_PREFIX));
    }

    @Test
    void shouldReturnProvidedValue() {
        BotConfig config = new BotConfig(Map.of("BOT_PREFIX", "?"));
        assertEquals("?", config.getOrDefault(ConfigKey.BOT_PREFIX));
    }

    @Test
    void shouldThrowForMissingRequiredKey() {
        BotConfig config = new BotConfig(Map.of());
        assertNull(config.get(ConfigKey.DISCORD_BOT_TOKEN));
    }
}
