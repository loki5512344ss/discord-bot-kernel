package kernel.loki.command;

public interface Command {
    String name();
    String description();
    default String usage() { return name(); }
    default String category() { return "General"; }
    void execute(CommandContext ctx);
}
