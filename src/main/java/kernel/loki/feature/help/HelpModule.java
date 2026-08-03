package kernel.loki.feature.help;

import com.google.inject.AbstractModule;
import com.google.inject.multibindings.Multibinder;
import kernel.loki.core.registry.CommandRegistrar;

public class HelpModule extends AbstractModule {
    @Override
    protected void configure() {
        Multibinder<CommandRegistrar> registrarBinder =
            Multibinder.newSetBinder(binder(), CommandRegistrar.class);
        registrarBinder.addBinding().to(HelpCommandRegistrar.class);
    }
}
