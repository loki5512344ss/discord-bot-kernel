package kernel.loki.core.registry;

@FunctionalInterface
public interface CommandRegistrar {
    void register(CommandRegistry registry);
}
