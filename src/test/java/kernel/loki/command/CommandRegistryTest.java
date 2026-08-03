package kernel.loki.command;

import static org.junit.jupiter.api.Assertions.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import kernel.loki.core.registry.CommandRegistry;

class CommandRegistryTest {
    private CommandRegistry registry;

    @BeforeEach
    void setUp() {
        registry = new CommandRegistry();
    }

    @Test
    void shouldRegisterAndRetrieveCommand() {
        Command cmd = new TestCommand("ping", "Ping command");
        registry.register(cmd);
        assertEquals(cmd, registry.get("ping"));
    }

    @Test
    void shouldBeCaseInsensitive() {
        Command cmd = new TestCommand("Ping", "Ping command");
        registry.register(cmd);
        assertNotNull(registry.get("PING"));
        assertNotNull(registry.get("ping"));
        assertNotNull(registry.get("Ping"));
    }

    @Test
    void shouldReturnNullForUnknownCommand() {
        assertNull(registry.get("nonexistent"));
    }

    @Test
    void shouldCountCommands() {
        registry.register(new TestCommand("a", ""));
        registry.register(new TestCommand("b", ""));
        assertEquals(2, registry.count());
    }

    private record TestCommand(String name, String description) implements Command {
        @Override
        public void execute(CommandContext ctx) {}
    }
}
