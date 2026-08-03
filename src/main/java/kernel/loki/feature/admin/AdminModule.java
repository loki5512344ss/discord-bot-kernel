package kernel.loki.feature.admin;

import com.google.inject.AbstractModule;
import com.google.inject.multibindings.Multibinder;
import kernel.loki.core.registry.CommandRegistrar;

public class AdminModule extends AbstractModule {
    @Override
    protected void configure() {
        Multibinder<CommandRegistrar> registrarBinder =
            Multibinder.newSetBinder(binder(), CommandRegistrar.class);
        registrarBinder.addBinding().to(AdminCommandRegistrar.class);
    }
}
