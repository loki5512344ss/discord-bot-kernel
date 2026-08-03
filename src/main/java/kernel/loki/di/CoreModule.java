package kernel.loki.di;

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Singleton;
import com.google.inject.multibindings.Multibinder;
import kernel.loki.config.BotConfig;
import kernel.loki.config.ConfigLoader;
import kernel.loki.core.EventDispatcher;
import kernel.loki.core.Lifecycle;
import kernel.loki.core.ListenerAggregator;
import kernel.loki.core.ShutdownManager;
import kernel.loki.core.registry.CommandRegistrar;
import kernel.loki.core.registry.CommandRegistry;
import kernel.loki.core.registry.Registrar;
import kernel.loki.core.security.CooldownManager;
import kernel.loki.core.security.RateLimiter;
import kernel.loki.database.Database;
import kernel.loki.database.DatabaseModule;
import kernel.loki.feature.admin.AdminModule;
import kernel.loki.feature.help.HelpModule;
import kernel.loki.feature.info.InfoModule;

public class CoreModule extends AbstractModule {
    @Override
    protected void configure() {
        bind(CommandRegistry.class).in(Singleton.class);
        bind(CooldownManager.class).in(Singleton.class);
        bind(RateLimiter.class).in(Singleton.class);
        bind(ListenerAggregator.class).in(Singleton.class);
        bind(ShutdownManager.class).in(Singleton.class);
        bind(Registrar.class).in(Singleton.class);

        Multibinder<Lifecycle> lifecycleBinder = Multibinder.newSetBinder(binder(), Lifecycle.class);
        lifecycleBinder.addBinding().to(CooldownManager.class);
        lifecycleBinder.addBinding().to(Database.class);
        lifecycleBinder.addBinding().to(EventDispatcher.class);

        install(new DatabaseModule());
        install(new HelpModule());
        install(new InfoModule());
        install(new AdminModule());
    }

    @Provides
    @Singleton
    BotConfig provideBotConfig() {
        return ConfigLoader.load();
    }

    @Provides
    @Singleton
    EventDispatcher provideEventDispatcher(
        BotConfig config,
        CommandRegistry registry,
        RateLimiter rateLimiter
    ) {
        return new EventDispatcher(config, registry, rateLimiter);
    }
}
