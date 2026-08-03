package kernel.loki.bootstrap;

import com.google.inject.Guice;
import com.google.inject.Injector;
import kernel.loki.core.Bot;
import kernel.loki.core.BotCluster;
import kernel.loki.core.BotInitializer;
import kernel.loki.core.bootstrap.BootstrapPhase;
import kernel.loki.di.CoreModule;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class App {
    private static final Logger logger = LoggerFactory.getLogger(App.class);

    public static void main(String[] args) {
        Thread shutdownHook = new Thread(() -> {
            LoggerDecorator.warn(logger, "Received shutdown signal");
            LoggerDecorator.warn(logger, "Performing graceful shutdown...");
        }, "ShutdownHook");

        Runtime.getRuntime().addShutdownHook(shutdownHook);

        try {
            LoggerDecorator.startup(logger, "Starting Bot Kernel v{}", "0.1.0");
            logPhase(BootstrapPhase.DI_INITIALIZE);

            Injector injector = Guice.createInjector(new CoreModule());

            BotInitializer initializer = injector.getInstance(BotInitializer.class);

            logPhase(BootstrapPhase.JDA_BUILD);
            BotCluster cluster = initializer.start();

            cluster.getByType(Bot.BotType.MAIN).ifPresent(bot -> {
                try {
                    bot.jda().awaitReady();
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    throw new RuntimeException("Interrupted while waiting for JDA ready", e);
                }
                logPhase(BootstrapPhase.COMPLETE);
            });

            LoggerDecorator.success(logger, "Bot Kernel is fully operational!");
        } catch (Exception e) {
            LoggerDecorator.error(logger, "Critical error during bootstrap", e);
            System.exit(1);
        }
    }

    private static void logPhase(BootstrapPhase phase) {
        LoggerDecorator.startup(logger, "Phase: {}", phase.name());
    }

    private App() {}
}
