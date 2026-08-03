package kernel.loki.core.security;

import static org.junit.jupiter.api.Assertions.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class RateLimiterTest {
    private RateLimiter limiter;

    @BeforeEach
    void setUp() {
        limiter = new RateLimiter();
    }

    @Test
    void shouldAllowFirstRequest() {
        RateLimitResult result = limiter.check("user1", "guild1");
        assertTrue(result.allowed());
    }

    @Test
    void shouldAllowMultipleRequestsUnderLimit() {
        for (int i = 0; i < 5; i++) {
            RateLimitResult result = limiter.check("user2", "guild1");
            assertTrue(result.allowed());
        }
    }

    @Test
    void shouldBlockAfterLimit() {
        for (int i = 0; i < 5; i++) {
            limiter.check("user3", "guild1");
        }
        RateLimitResult result = limiter.check("user3", "guild1");
        assertFalse(result.allowed());
        assertTrue(result.waitSeconds() > 0);
    }

    @Test
    void shouldHaveSeparateCountersPerUser() {
        for (int i = 0; i < 10; i++) {
            limiter.check("userA", "guild1");
        }
        RateLimitResult result = limiter.check("userB", "guild1");
        assertTrue(result.allowed());
    }
}
