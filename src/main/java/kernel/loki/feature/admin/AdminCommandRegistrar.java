package kernel.loki.feature.admin;

import com.google.inject.Inject;
import kernel.loki.core.registry.CommandRegistrar;
import kernel.loki.core.registry.CommandRegistry;
import kernel.loki.feature.admin.command.ShutdownCommand;

public class AdminCommandRegistrar implements CommandRegistrar {
    private final ShutdownCommand shutdown;

    @Inject
    public AdminCommandRegistrar(ShutdownCommand shutdown) {
        this.shutdown = shutdown;
    }

    @Override
    public void register(CommandRegistry registry) {
        registry.register(shutdown);
    }
}
