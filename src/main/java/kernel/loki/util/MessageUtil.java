package kernel.loki.util;

import java.util.concurrent.TimeUnit;
import net.dv8tion.jda.api.entities.Message;
import net.dv8tion.jda.api.entities.channel.middleman.MessageChannel;

public class MessageUtil {
    private MessageUtil() {}

    public static void deleteAfter(Message message, long seconds) {
        message.delete().queueAfter(seconds, TimeUnit.SECONDS);
    }

    public static void sendAndDelete(MessageChannel channel, String content, long seconds) {
        channel.sendMessage(content).queue(msg -> deleteAfter(msg, seconds));
    }

    public static String sanitize(String input) {
        return input.replace("@everyone", "@\u200Beveryone")
            .replace("@here", "@\u200Bhere");
    }

    public static String truncate(String input, int maxLength) {
        if (input == null) return null;
        if (input.length() <= maxLength) return input;
        return input.substring(0, maxLength - 3) + "...";
    }
}
