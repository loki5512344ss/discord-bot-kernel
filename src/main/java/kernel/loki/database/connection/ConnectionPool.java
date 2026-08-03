package kernel.loki.database.connection;

import java.sql.Connection;

public interface ConnectionPool {
    Connection getConnection();
    void close();
}
