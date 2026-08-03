package kernel.loki.core;

import net.dv8tion.jda.api.JDA;

public record Bot(String id, JDA jda, BotType type) {
    public enum BotType {
        MAIN, MUSIC, RADIO
    }
}
