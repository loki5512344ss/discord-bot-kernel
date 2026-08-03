package kernel.loki.core;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import net.dv8tion.jda.api.JDA;

public class BotCluster {
    private final Map<String, Bot> bots = new HashMap<>();

    public BotCluster(List<Bot> botList) {
        for (Bot bot : botList) {
            bots.put(bot.id(), bot);
        }
    }

    public void add(Bot bot) {
        bots.put(bot.id(), bot);
    }

    public Optional<Bot> get(String id) {
        return Optional.ofNullable(bots.get(id));
    }

    public Optional<Bot> getByType(Bot.BotType type) {
        return bots.values().stream()
            .filter(b -> b.type() == type)
            .findFirst();
    }

    public JDA main() {
        return getByType(Bot.BotType.MAIN)
            .map(Bot::jda)
            .orElseThrow(() -> new IllegalStateException("Main bot not initialized"));
    }

    public List<Bot> all() {
        return List.copyOf(bots.values());
    }

    public int count() {
        return bots.size();
    }
}
