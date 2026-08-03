package kernel.loki.database.migration.migrator;

import java.sql.Connection;
import java.sql.SQLException;
import java.sql.Statement;
import kernel.loki.database.migration.Migration;

public class V1__InitialSchema implements Migration {
    @Override
    public int version() {
        return 1;
    }

    @Override
    public String description() {
        return "Create core tables";
    }

    @Override
    public void migrate(Connection conn) throws SQLException {
        try (Statement stmt = conn.createStatement()) {
            stmt.execute("""
                CREATE TABLE IF NOT EXISTS guild_config (
                    guild_id TEXT PRIMARY KEY,
                    prefix TEXT DEFAULT '!',
                    settings TEXT DEFAULT '{}',
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
            """);

            stmt.execute("""
                CREATE TABLE IF NOT EXISTS user_data (
                    user_id TEXT NOT NULL,
                    guild_id TEXT NOT NULL,
                    key TEXT NOT NULL,
                    value TEXT,
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    PRIMARY KEY (user_id, guild_id, key)
                )
            """);

            stmt.execute("""
                CREATE TABLE IF NOT EXISTS metrics (
                    command_name TEXT NOT NULL,
                    call_count INTEGER DEFAULT 0,
                    total_time_ms INTEGER DEFAULT 0,
                    error_count INTEGER DEFAULT 0,
                    last_called_at TEXT,
                    PRIMARY KEY (command_name)
                )
            """);
        }
    }
}
