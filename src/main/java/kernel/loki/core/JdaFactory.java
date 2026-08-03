package kernel.loki.core;

import java.util.Arrays;
import net.dv8tion.jda.api.JDA;
import net.dv8tion.jda.api.JDABuilder;
import net.dv8tion.jda.api.entities.Activity;
import net.dv8tion.jda.api.requests.GatewayIntent;
import net.dv8tion.jda.api.utils.cache.CacheFlag;

public class JdaFactory {
    private JdaFactory() {}

    public static JDABuilder createBuilder(String token, String activity, GatewayIntent... intents) {
        JDABuilder builder = JDABuilder.createDefault(token)
            .setActivity(Activity.customStatus(activity))
            .enableIntents(Arrays.asList(intents))
            .enableCache(CacheFlag.VOICE_STATE)
            .setAutoReconnect(true);

        return builder;
    }

    public static JDA awaitJda(JDA jda) {
        try {
            return jda.awaitReady();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new RuntimeException("Interrupted while waiting for JDA ready", e);
        }
    }
}
