package kernel.loki.core.security;

import com.github.benmanes.caffeine.cache.Cache;
import com.github.benmanes.caffeine.cache.Caffeine;
import java.util.concurrent.TimeUnit;
import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.core.Lifecycle;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Singleton
public class CooldownManager implements Lifecycle {
    private static final Logger logger = LoggerFactory.getLogger(CooldownManager.class);
    private Cache<String, Long> cooldowns;

    @Inject
    public CooldownManager() {
        this.cooldowns = Caffeine.newBuilder()
            .expireAfterWrite(1, TimeUnit.HOURS)
            .build();
    }

    public boolean isOnCooldown(String key) {
        Long expiry = cooldowns.getIfPresent(key);
        return expiry != null && System.currentTimeMillis() < expiry;
    }

    public long remaining(String key) {
        Long expiry = cooldowns.getIfPresent(key);
        if (expiry == null) return 0;
        long rem = expiry - System.currentTimeMillis();
        return Math.max(0, rem);
    }

    public void set(String key, long durationMs) {
        cooldowns.put(key, System.currentTimeMillis() + durationMs);
    }

    @Override
    public void shutdown() {
        LoggerDecorator.database(logger, "Cooldown cache cleared");
        cooldowns.invalidateAll();
    }
}
