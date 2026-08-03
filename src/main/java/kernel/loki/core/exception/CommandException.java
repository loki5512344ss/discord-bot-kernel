package kernel.loki.core.exception;

public class CommandException extends RuntimeException {
    private final String commandName;

    public CommandException(String commandName, String message) {
        super(message);
        this.commandName = commandName;
    }

    public CommandException(String commandName, String message, Throwable cause) {
        super(message, cause);
        this.commandName = commandName;
    }

    public String commandName() { return commandName; }
}
