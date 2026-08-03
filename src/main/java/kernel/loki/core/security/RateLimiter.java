package kernel.loki.core.security;

import com.github.benmanes.caffeine.cache.Cache;
import com.github.benmanes.caffeine.cache.Caffeine;
import java.util.concurrent.TimeUnit;
import com.google.inject.Inject;
import com.google.inject.Singleton;

@Singleton
public class RateLimiter {
    private static final int MAX_REQUESTS = 5;
    private static final int WINDOW_SECONDS = 10;

    private final Cache<String, RequestCounter> cache;

    @Inject
    public RateLimiter() {
        this.cache = Caffeine.newBuilder()
            .expireAfterWrite(WINDOW_SECONDS, TimeUnit.SECONDS)
            .build();
    }

    public RateLimitResult check(String userId, String guildId) {
        String key = userId + ":" + guildId;
        RequestCounter counter = cache.get(key, k -> new RequestCounter());
        return counter.tryAcquire();
    }

    private static class RequestCounter {
        private int count;
        private long windowStart;

        RequestCounter() {
            this.count = 0;
            this.windowStart = System.nanoTime();
        }

        synchronized RateLimitResult tryAcquire() {
            long now = System.nanoTime();
            if (now - windowStart > TimeUnit.SECONDS.toNanos(WINDOW_SECONDS)) {
                count = 0;
                windowStart = now;
            }
            count++;
            if (count > MAX_REQUESTS) {
                long elapsed = TimeUnit.NANOSECONDS.toSeconds(now - windowStart);
                long wait = WINDOW_SECONDS - elapsed;
                return new RateLimitResult(false, Math.max(1, wait));
            }
            return RateLimitResult.ALLOWED;
        }
    }
}
