package kernel.loki.command;

public enum CommandCategory {
    ADMIN("Admin"),
    HELP("Help"),
    INFO("Info"),
    MODERATION("Moderation"),
    UTILITY("Utility"),
    FUN("Fun"),
    ECONOMY("Economy"),
    MUSIC("Music"),
    GENERAL("General");

    private final String display;

    CommandCategory(String display) {
        this.display = display;
    }

    public String display() { return display; }
}
