package kernel.loki.core.security;

public record RateLimitResult(boolean allowed, long waitSeconds) {
    public static final RateLimitResult ALLOWED = new RateLimitResult(true, 0);
}
