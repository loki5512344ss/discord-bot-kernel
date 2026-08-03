package kernel.loki.database.connection;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.function.Function;

public class ConnectionFactory {
    private final ConnectionPool pool;

    public ConnectionFactory(ConnectionPool pool) {
        this.pool = pool;
    }

    public <T> T withConnection(Function<Connection, T> callback) {
        try (Connection conn = pool.getConnection()) {
            return callback.apply(conn);
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    public void withConnectionVoid(ThrowingConsumer<Connection> callback) {
        try (Connection conn = pool.getConnection()) {
            callback.accept(conn);
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    @FunctionalInterface
    public interface ThrowingConsumer<T> {
        void accept(T t) throws SQLException;
    }
}
