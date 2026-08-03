package kernel.loki.feature.info;

import com.google.inject.AbstractModule;
import com.google.inject.multibindings.Multibinder;
import kernel.loki.core.registry.CommandRegistrar;

public class InfoModule extends AbstractModule {
    @Override
    protected void configure() {
        Multibinder<CommandRegistrar> registrarBinder =
            Multibinder.newSetBinder(binder(), CommandRegistrar.class);
        registrarBinder.addBinding().to(InfoCommandRegistrar.class);
    }
}
