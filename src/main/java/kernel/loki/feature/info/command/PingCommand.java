package kernel.loki.feature.info.command;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.command.Command;
import kernel.loki.command.CommandCategory;
import kernel.loki.command.CommandContext;
import kernel.loki.embed.EmbedTemplates;

@Singleton
public class PingCommand implements Command {
    @Inject
    public PingCommand() {}

    @Override
    public String name() { return "ping"; }

    @Override
    public String description() { return "Check bot latency"; }

    @Override
    public String category() { return CommandCategory.INFO.display(); }

    @Override
    public void execute(CommandContext ctx) {
        long start = System.nanoTime();
        ctx.channel().sendMessage("Pong!").flatMap(msg -> {
            long elapsed = (System.nanoTime() - start) / 1_000_000;
            msg.editMessageEmbeds(
                EmbedTemplates.info("Pong!",
                    "Gateway: " + ctx.event().getJDA().getGatewayPing() + "ms\n"
                    + "Roundtrip: " + elapsed + "ms"))
                .queue();
            return null;
        }).queue();
    }
}
