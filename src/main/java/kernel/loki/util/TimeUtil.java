package kernel.loki.util;

import java.time.Duration;
import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;

public class TimeUtil {
    private static final DateTimeFormatter FORMATTER =
        DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss")
            .withZone(ZoneId.systemDefault());

    private TimeUtil() {}

    public static String format(Instant instant) {
        return FORMATTER.format(instant);
    }

    public static String formatDuration(long millis) {
        Duration duration = Duration.ofMillis(millis);
        long hours = duration.toHours();
        long minutes = duration.toMinutes() % 60;
        long seconds = duration.getSeconds() % 60;

        if (hours > 0) {
            return String.format("%dh %dm %ds", hours, minutes, seconds);
        } else if (minutes > 0) {
            return String.format("%dm %ds", minutes, seconds);
        } else {
            return String.format("%ds", seconds);
        }
    }

    public static long parseDuration(String input) {
        input = input.toLowerCase().trim();
        long total = 0;
        StringBuilder num = new StringBuilder();

        for (char c : input.toCharArray()) {
            if (Character.isDigit(c)) {
                num.append(c);
            } else {
                if (num.isEmpty()) continue;
                long value = Long.parseLong(num.toString());
                switch (c) {
                    case 's' -> total += value * 1000;
                    case 'm' -> total += value * 60_000;
                    case 'h' -> total += value * 3_600_000;
                    case 'd' -> total += value * 86_400_000;
                }
                num.setLength(0);
            }
        }
        return total;
    }
}
