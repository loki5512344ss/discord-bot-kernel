package kernel.loki.core;

import java.util.Arrays;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import kernel.loki.command.Command;
import kernel.loki.command.CommandContext;
import kernel.loki.config.BotConfig;
import kernel.loki.core.exception.CommandExceptionHandler;
import kernel.loki.core.registry.CommandRegistry;
import kernel.loki.core.security.RateLimitResult;
import kernel.loki.core.security.RateLimiter;
import kernel.loki.embed.EmbedTemplates;
import kernel.loki.util.LoggerDecorator;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class EventDispatcher implements Lifecycle {
    private static final Logger logger = LoggerFactory.getLogger(EventDispatcher.class);
    private final BotConfig config;
    private final CommandRegistry registry;
    private final ExecutorService executor;
    private final RateLimiter rateLimiter;

    public EventDispatcher(BotConfig config, CommandRegistry registry, RateLimiter rateLimiter) {
        this.config = config;
        this.registry = registry;
        this.rateLimiter = rateLimiter;
        this.executor = Executors.newVirtualThreadPerTaskExecutor();
    }

    @Override
    public void shutdown() {
        LoggerDecorator.bot(logger, "Shutting down EventDispatcher...");
        executor.shutdown();
        try {
            if (!executor.awaitTermination(10, TimeUnit.SECONDS)) {
                LoggerDecorator.warn(logger, "Executor did not terminate in 10s");
                executor.shutdownNow();
            }
        } catch (InterruptedException e) {
            executor.shutdownNow();
            Thread.currentThread().interrupt();
        }
        LoggerDecorator.success(logger, "EventDispatcher shut down");
    }

    public void dispatch(MessageReceivedEvent event) {
        if (event.getAuthor().isBot()) return;

        String raw = event.getMessage().getContentRaw();
        String prefix = config.getPrefix();
        if (!raw.startsWith(prefix)) return;

        String[] parts = raw.substring(prefix.length()).trim().split("\\s+");
        if (parts.length == 0) return;

        String name = parts[0].toLowerCase();
        List<String> args = parts.length > 1
            ? Arrays.asList(Arrays.copyOfRange(parts, 1, parts.length))
            : List.of();

        Command command = registry.get(name);
        if (command == null) {
            event.getChannel().sendMessageEmbeds(
                EmbedTemplates.error("Command not found",
                    "`" + name + "` does not exist. Use `" + prefix + "help`"))
                .queue();
            return;
        }

        RateLimitResult rl = rateLimiter.check(event.getAuthor().getId(), event.getGuild().getId());
        if (!rl.allowed()) {
            event.getChannel().sendMessageEmbeds(
                EmbedTemplates.error("Too fast!",
                    "Please wait " + rl.waitSeconds() + " seconds."))
                .queue();
            return;
        }

        executor.submit(() -> {
            long start = System.nanoTime();
            try {
                CommandContext ctx = new CommandContext(event, args);
                command.execute(ctx);
                long elapsed = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - start);
                LoggerDecorator.command(logger, name, event.getAuthor().getName(), "{}ms", elapsed);
            } catch (Exception e) {
                CommandExceptionHandler.handle(e, event.getChannel(), name);
            }
        });
    }
}
