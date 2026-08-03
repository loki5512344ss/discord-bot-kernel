package kernel.loki.database;

public class DatabaseSchema {
    private DatabaseSchema() {}

    public static final class GuildConfig {
        public static final String TABLE = "guild_config";
        public static final String GUILD_ID = "guild_id";
        public static final String PREFIX = "prefix";
        public static final String SETTINGS = "settings";
        public static final String UPDATED_AT = "updated_at";

        private GuildConfig() {}
    }

    public static final class UserData {
        public static final String TABLE = "user_data";
        public static final String USER_ID = "user_id";
        public static final String GUILD_ID = "guild_id";
        public static final String KEY = "key";
        public static final String VALUE = "value";

        private UserData() {}
    }

    public static final class Metrics {
        public static final String TABLE = "metrics";
        public static final String COMMAND_NAME = "command_name";
        public static final String CALL_COUNT = "call_count";
        public static final String TOTAL_TIME_MS = "total_time_ms";
        public static final String ERROR_COUNT = "error_count";
        public static final String LAST_CALLED_AT = "last_called_at";

        private Metrics() {}
    }
}
