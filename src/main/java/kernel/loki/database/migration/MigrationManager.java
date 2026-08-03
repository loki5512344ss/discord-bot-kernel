package kernel.loki.database.migration;

import java.sql.Connection;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.ServiceLoader;
import kernel.loki.database.Database;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class MigrationManager {
    private static final Logger logger = LoggerFactory.getLogger(MigrationManager.class);
    private static final int TARGET_VERSION = Integer.MAX_VALUE;
    private final Database database;
    private final List<Migration> migrations = new ArrayList<>();

    public MigrationManager(Database database) {
        this.database = database;
        discoverMigrations();
    }

    private void discoverMigrations() {
        ServiceLoader.load(Migration.class)
            .forEach(migrations::add);

        try {
            Class.forName("kernel.loki.database.migration.migrator.V1__InitialSchema");
            migrations.add(new kernel.loki.database.migration.migrator.V1__InitialSchema());
        } catch (ClassNotFoundException ignored) {}

        migrations.sort(Comparator.comparingInt(Migration::version));
    }

    public void migrate() {
        ensureSchemaTable();
        int currentVersion = getCurrentVersion();

        for (Migration migration : migrations) {
            if (migration.version() > currentVersion) {
                LoggerDecorator.database(logger, "Applying migration v{}: {}",
                    migration.version(), migration.description());
                try (Connection conn = database.getConnection()) {
                    conn.setAutoCommit(false);
                    try {
                        migration.migrate(conn);
                        setVersion(conn, migration.version());
                        conn.commit();
                        LoggerDecorator.success(logger, "Migration v{} applied", migration.version());
                    } catch (SQLException e) {
                        conn.rollback();
                        throw e;
                    }
                } catch (SQLException e) {
                    throw new RuntimeException("Migration v" + migration.version() + " failed", e);
                }
            }
        }
    }

    private void ensureSchemaTable() {
        try (Connection conn = database.getConnection();
             Statement stmt = conn.createStatement()) {
            stmt.execute("""
                CREATE TABLE IF NOT EXISTS schema_version (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
            """);
        } catch (SQLException e) {
            throw new RuntimeException("Failed to create schema_version table", e);
        }
    }

    private int getCurrentVersion() {
        try (Connection conn = database.getConnection();
             Statement stmt = conn.createStatement();
             ResultSet rs = stmt.executeQuery("SELECT COALESCE(MAX(version), 0) FROM schema_version")) {
            return rs.next() ? rs.getInt(1) : 0;
        } catch (SQLException e) {
            return 0;
        }
    }

    private void setVersion(Connection conn, int version) throws SQLException {
        try (Statement stmt = conn.createStatement()) {
            stmt.execute("INSERT INTO schema_version (version) VALUES (" + version + ")");
        }
    }
}
