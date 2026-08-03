package kernel.loki.database;

import com.google.inject.AbstractModule;
import com.google.inject.Provides;
import com.google.inject.Singleton;

public class DatabaseModule extends AbstractModule {
    @Override
    protected void configure() {
        // Database itself is bound in CoreModule via Multibinder<Lifecycle>
    }

    @Provides
    @Singleton
    Database provideDatabase(kernel.loki.config.BotConfig config) {
        return new Database(config);
    }
}
