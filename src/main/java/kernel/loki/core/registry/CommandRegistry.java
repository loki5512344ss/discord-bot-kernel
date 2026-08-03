package kernel.loki.core.registry;

import java.util.Collection;
import java.util.Collections;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;
import kernel.loki.command.Command;

public class CommandRegistry {
    private final ConcurrentMap<String, Command> commands = new ConcurrentHashMap<>();

    public void register(Command command) {
        commands.put(command.name().toLowerCase(), command);
    }

    public Command get(String name) {
        return commands.get(name.toLowerCase());
    }

    public Collection<Command> all() {
        return Collections.unmodifiableCollection(commands.values());
    }

    public int count() {
        return commands.size();
    }
}
