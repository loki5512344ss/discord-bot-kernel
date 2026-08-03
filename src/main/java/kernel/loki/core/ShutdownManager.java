package kernel.loki.core;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Singleton
public class ShutdownManager {
    private static final Logger logger = LoggerFactory.getLogger(ShutdownManager.class);
    private final LifecycleManager lifecycleManager;

    @Inject
    public ShutdownManager(LifecycleManager lifecycleManager) {
        this.lifecycleManager = lifecycleManager;
    }

    public void shutdown(BotCluster cluster) {
        LoggerDecorator.warn(logger, "Shutting down bot cluster...");

        lifecycleManager.shutdownAll();

        cluster.all().forEach(bot -> {
            bot.jda().shutdown();
            LoggerDecorator.bot(logger, "JDA {} ({}) shut down", bot.id(), bot.type());
        });

        LoggerDecorator.success(logger, "Bot cluster fully shut down");
        LoggerDecorator.warn(logger, "Goodbye!");
    }
}
