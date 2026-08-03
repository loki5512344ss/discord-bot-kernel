package kernel.loki.feature.help.command;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.command.Command;
import kernel.loki.command.CommandCategory;
import kernel.loki.command.CommandContext;
import kernel.loki.config.BotConfig;
import kernel.loki.core.registry.CommandRegistry;
import kernel.loki.embed.EmbedTemplates;

@Singleton
public class HelpCommand implements Command {
    private final BotConfig config;
    private final CommandRegistry registry;

    @Inject
    public HelpCommand(BotConfig config, CommandRegistry registry) {
        this.config = config;
        this.registry = registry;
    }

    @Override
    public String name() { return "help"; }

    @Override
    public String description() { return "Show available commands"; }

    @Override
    public String category() { return CommandCategory.HELP.display(); }

    @Override
    public void execute(CommandContext ctx) {
        String prefix = config.getPrefix();
        ctx.channel().sendMessageEmbeds(
            EmbedTemplates.commandList(prefix, registry.all()))
            .queue();
    }
}
