package kernel.loki.core.registry;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import java.util.Set;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Singleton
public class Registrar {
    private static final Logger logger = LoggerFactory.getLogger(Registrar.class);
    private final CommandRegistry registry;
    private final Set<CommandRegistrar> registrars;

    @Inject
    public Registrar(CommandRegistry registry, Set<CommandRegistrar> registrars) {
        this.registry = registry;
        this.registrars = registrars;
    }

    public void registerAll() {
        for (CommandRegistrar registrar : registrars) {
            registrar.register(registry);
        }
        LoggerDecorator.success(logger, "Registered {} prefix commands", registry.count());
    }
}
