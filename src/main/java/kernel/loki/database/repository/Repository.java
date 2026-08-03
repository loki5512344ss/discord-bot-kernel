package kernel.loki.database.repository;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

public interface Repository<T, ID> {
    Optional<T> findById(ID id);
    void save(T entity);
    void delete(ID id);
}
