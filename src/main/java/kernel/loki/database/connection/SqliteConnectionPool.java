package kernel.loki.database.connection;

import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import java.sql.Connection;
import java.sql.SQLException;

public class SqliteConnectionPool implements ConnectionPool {
    private final HikariDataSource dataSource;

    public SqliteConnectionPool(String dbPath) {
        HikariConfig config = new HikariConfig();
        config.setJdbcUrl("jdbc:sqlite:" + dbPath);
        config.setMaximumPoolSize(4);
        config.setConnectionTimeout(10000);
        config.setIdleTimeout(0);
        config.setMaxLifetime(600000);
        config.setPoolName("KernelDB");
        this.dataSource = new HikariDataSource(config);
    }

    @Override
    public Connection getConnection() {
        try {
            return dataSource.getConnection();
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public void close() {
        if (dataSource != null && !dataSource.isClosed()) {
            dataSource.close();
        }
    }
}
