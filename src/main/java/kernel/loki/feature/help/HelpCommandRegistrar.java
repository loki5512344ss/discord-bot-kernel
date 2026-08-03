package kernel.loki.feature.help;

import com.google.inject.Inject;
import kernel.loki.core.registry.CommandRegistrar;
import kernel.loki.core.registry.CommandRegistry;
import kernel.loki.feature.help.command.HelpCommand;

public class HelpCommandRegistrar implements CommandRegistrar {
    private final HelpCommand command;

    @Inject
    public HelpCommandRegistrar(HelpCommand command) {
        this.command = command;
    }

    @Override
    public void register(CommandRegistry registry) {
        registry.register(command);
    }
}
