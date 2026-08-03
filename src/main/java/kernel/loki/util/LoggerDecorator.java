package kernel.loki.util;

import org.slf4j.Logger;
import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

public class LoggerDecorator {
    private static final Marker STARTUP = MarkerFactory.getMarker("STARTUP");
    private static final Marker BOT = MarkerFactory.getMarker("BOT");
    private static final Marker DATABASE = MarkerFactory.getMarker("DATABASE");
    private static final Marker COMMAND = MarkerFactory.getMarker("COMMAND");
    private static final Marker SUCCESS = MarkerFactory.getMarker("SUCCESS");

    private LoggerDecorator() {}

    public static void startup(Logger log, String msg, Object... args) {
        log.info(STARTUP, msg, args);
    }

    public static void bot(Logger log, String msg, Object... args) {
        log.info(BOT, msg, args);
    }

    public static void database(Logger log, String msg, Object... args) {
        log.info(DATABASE, msg, args);
    }

    public static void command(Logger log, String cmd, String user, String msg, Object... args) {
        log.info(COMMAND, "[{}] by {}: {}", cmd, user, String.format(msg, args));
    }

    public static void success(Logger log, String msg, Object... args) {
        log.info(SUCCESS, msg, args);
    }

    public static void warn(Logger log, String msg, Object... args) {
        log.warn(msg, args);
    }

    public static void error(Logger log, String msg, Object... args) {
        log.error(msg, args);
    }

    public static void error(Logger log, String msg, Throwable t) {
        log.error(msg, t);
    }
}
