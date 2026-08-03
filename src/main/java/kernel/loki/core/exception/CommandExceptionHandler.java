package kernel.loki.core.exception;

import kernel.loki.embed.EmbedTemplates;
import kernel.loki.util.LoggerDecorator;
import net.dv8tion.jda.api.entities.channel.middleman.MessageChannel;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CommandExceptionHandler {
    private static final Logger logger = LoggerFactory.getLogger(CommandExceptionHandler.class);

    private CommandExceptionHandler() {}

    public static void handle(Exception e, MessageChannel channel, String commandName) {
        if (e instanceof CommandException ce) {
            channel.sendMessageEmbeds(
                EmbedTemplates.error("Command Error", ce.getMessage()))
                .queue();
            LoggerDecorator.warn(logger, "Command {}: {}", commandName, ce.getMessage());
        } else {
            channel.sendMessageEmbeds(
                EmbedTemplates.error("Internal Error",
                    "An unexpected error occurred. Please try again later."))
                .queue();
            LoggerDecorator.error(logger, "Unhandled exception in command {}", commandName, e);
        }
    }
}
