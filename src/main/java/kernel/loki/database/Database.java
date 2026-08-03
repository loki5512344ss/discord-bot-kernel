package kernel.loki.database;

import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import java.sql.Connection;
import java.sql.SQLException;
import java.sql.Statement;
import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.config.BotConfig;
import kernel.loki.core.Lifecycle;
import kernel.loki.database.connection.ConnectionPool;
import kernel.loki.database.connection.SqliteConnectionPool;
import kernel.loki.database.migration.MigrationManager;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Singleton
public class Database implements Lifecycle {
    private static final Logger logger = LoggerFactory.getLogger(Database.class);
    private final BotConfig config;
    private ConnectionPool pool;
    private MigrationManager migrationManager;

    @Inject
    public Database(BotConfig config) {
        this.config = config;
    }

    @Override
    public void start() {
        LoggerDecorator.database(logger, "Connecting to database: {}", config.getDbPath());
        this.pool = new SqliteConnectionPool(config.getDbPath());
        this.migrationManager = new MigrationManager(this);

        try (Connection conn = pool.getConnection();
             Statement stmt = conn.createStatement()) {
            stmt.execute("PRAGMA foreign_keys = ON");
            stmt.execute("PRAGMA journal_mode = WAL");
            stmt.execute("PRAGMA busy_timeout = 5000");
        } catch (SQLException e) {
            throw new RuntimeException("Failed to configure database", e);
        }

        migrationManager.migrate();
        LoggerDecorator.success(logger, "Database ready");
    }

    public Connection getConnection() {
        return pool.getConnection();
    }

    public MigrationManager migrations() {
        return migrationManager;
    }

    @Override
    public void shutdown() {
        if (pool != null) {
            pool.close();
            LoggerDecorator.database(logger, "Database connection pool closed");
        }
    }
}
