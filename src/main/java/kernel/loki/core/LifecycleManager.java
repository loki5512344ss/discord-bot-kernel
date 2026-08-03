package kernel.loki.core;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import java.util.Set;
import kernel.loki.util.LoggerDecorator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Singleton
public class LifecycleManager {
    private static final Logger logger = LoggerFactory.getLogger(LifecycleManager.class);
    private final Set<Lifecycle> components;

    @Inject
    public LifecycleManager(Set<Lifecycle> components) {
        this.components = components;
    }

    public void startAll() {
        LoggerDecorator.startup(logger, "Starting {} lifecycle components", components.size());
        for (Lifecycle component : components) {
            try {
                component.start();
            } catch (Exception e) {
                LoggerDecorator.error(logger, "Failed to start {}", component.getClass().getSimpleName(), e);
            }
        }
        LoggerDecorator.success(logger, "All lifecycle components started");
    }

    public void shutdownAll() {
        LoggerDecorator.bot(logger, "Shutting down {} lifecycle components", components.size());
        for (Lifecycle component : components) {
            try {
                component.shutdown();
            } catch (Exception e) {
                LoggerDecorator.error(logger, "Failed to shutdown {}", component.getClass().getSimpleName(), e);
            }
        }
        LoggerDecorator.success(logger, "All lifecycle components shut down");
    }
}
