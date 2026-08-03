package kernel.loki.command;

public sealed interface CommandResult {
    record Success(String message) implements CommandResult {}
    record Error(String reason) implements CommandResult {}
    record Cooldown(long waitSeconds) implements CommandResult {}
    record NotFound(String name) implements CommandResult {}
}
