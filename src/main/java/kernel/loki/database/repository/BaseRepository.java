package kernel.loki.database.repository;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.function.Function;
import com.google.inject.Inject;
import kernel.loki.database.Database;

public abstract class BaseRepository {
    protected final Database database;

    @Inject
    protected BaseRepository(Database database) {
        this.database = database;
    }

    protected <T> T withConnection(Function<Connection, T> callback) {
        try (Connection conn = database.getConnection()) {
            return callback.apply(conn);
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    protected void withConnectionVoid(ThrowingConsumer<Connection> callback) {
        try (Connection conn = database.getConnection()) {
            callback.accept(conn);
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    protected void setString(PreparedStatement stmt, int index, String value) throws SQLException {
        if (value != null) {
            stmt.setString(index, value);
        } else {
            stmt.setNull(index, java.sql.Types.VARCHAR);
        }
    }

    protected String getString(ResultSet rs, String column) throws SQLException {
        String val = rs.getString(column);
        return rs.wasNull() ? null : val;
    }

    @FunctionalInterface
    protected interface ThrowingConsumer<T> {
        void accept(T t) throws SQLException;
    }
}
