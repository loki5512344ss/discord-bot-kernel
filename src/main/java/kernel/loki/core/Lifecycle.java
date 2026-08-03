package kernel.loki.core;

public interface Lifecycle {
    default void start() {}
    default void shutdown() {}
}
