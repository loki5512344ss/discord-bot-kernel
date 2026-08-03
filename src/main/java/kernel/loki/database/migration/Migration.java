package kernel.loki.database.migration;

import java.sql.Connection;
import java.sql.SQLException;

public interface Migration {
    int version();
    String description();
    void migrate(Connection conn) throws SQLException;
}
