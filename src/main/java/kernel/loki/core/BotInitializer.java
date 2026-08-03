package kernel.loki.core;

import com.google.inject.Inject;
import java.util.ArrayList;
import java.util.List;
import kernel.loki.config.BotConfig;
import kernel.loki.core.registry.Registrar;
import kernel.loki.util.LoggerDecorator;
import net.dv8tion.jda.api.JDA;
import net.dv8tion.jda.api.JDABuilder;
import net.dv8tion.jda.api.requests.GatewayIntent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class BotInitializer {
    private static final Logger logger = LoggerFactory.getLogger(BotInitializer.class);
    private static final GatewayIntent[] DEFAULT_INTENTS = {
        GatewayIntent.GUILD_MESSAGES,
        GatewayIntent.MESSAGE_CONTENT,
        GatewayIntent.GUILD_MEMBERS,
        GatewayIntent.GUILD_VOICE_STATES,
        GatewayIntent.GUILD_MESSAGE_REACTIONS
    };

    private final BotConfig config;
    private final Registrar registrar;
    private final ListenerAggregator listeners;
    private final LifecycleManager lifecycleManager;
    private final ShutdownManager shutdownManager;
    private final EventDispatcher dispatcher;

    @Inject
    public BotInitializer(
        BotConfig config,
        Registrar registrar,
        ListenerAggregator listeners,
        LifecycleManager lifecycleManager,
        ShutdownManager shutdownManager,
        EventDispatcher dispatcher
    ) {
        this.config = config;
        this.registrar = registrar;
        this.listeners = listeners;
        this.lifecycleManager = lifecycleManager;
        this.shutdownManager = shutdownManager;
        this.dispatcher = dispatcher;
    }

    public BotCluster start() {
        LoggerDecorator.startup(logger, "Initializing bot kernel...");

        registrar.registerAll();
        lifecycleManager.startAll();

        JDA mainJda = buildMainJda();
        List<Bot> botList = new ArrayList<>();
        botList.add(new Bot("main", mainJda, Bot.BotType.MAIN));

        BotCluster cluster = new BotCluster(botList);

        Runtime.getRuntime().addShutdownHook(
            new Thread(() -> shutdownManager.shutdown(cluster), "ShutdownHook"));

        LoggerDecorator.success(logger, "Bot kernel initialized with {} bot(s)", botList.size());
        return cluster;
    }

    private JDA buildMainJda() {
        JDABuilder builder = JdaFactory.createBuilder(
            config.getToken(),
            config.getActivity(),
            DEFAULT_INTENTS
        );

        for (Object listener : listeners.all()) {
            builder.addEventListeners(listener);
        }

        return JdaFactory.awaitJda(builder.build());
    }
}
