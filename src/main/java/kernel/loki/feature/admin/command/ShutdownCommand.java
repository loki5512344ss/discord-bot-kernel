package kernel.loki.feature.admin.command;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.command.Command;
import kernel.loki.command.CommandCategory;
import kernel.loki.command.CommandContext;
import kernel.loki.config.BotConfig;
import kernel.loki.embed.EmbedTemplates;

@Singleton
public class ShutdownCommand implements Command {
    private final BotConfig config;

    @Inject
    public ShutdownCommand(BotConfig config) {
        this.config = config;
    }

    @Override
    public String name() { return "shutdown"; }

    @Override
    public String description() { return "Shutdown the bot (admin only)"; }

    @Override
    public String category() { return CommandCategory.ADMIN.display(); }

    @Override
    public void execute(CommandContext ctx) {
        String adminId = config.getSuperAdminId();
        if (adminId == null || !adminId.equals(ctx.author().getId())) {
            ctx.channel().sendMessageEmbeds(
                EmbedTemplates.error("Access Denied", "You are not authorized to use this command."))
                .queue();
            return;
        }

        ctx.channel().sendMessageEmbeds(
            EmbedTemplates.warning("Shutting down", "Bot will shut down now."))
            .queue(msg -> {
                ctx.event().getJDA().shutdown();
                System.exit(0);
            });
    }
}
