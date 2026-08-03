package kernel.loki.embed;

import net.dv8tion.jda.api.EmbedBuilder;
import net.dv8tion.jda.api.entities.MessageEmbed;
import net.dv8tion.jda.api.entities.User;

public class EmbedTemplates {
    private EmbedTemplates() {}

    public static MessageEmbed error(String title, String description) {
        return EmbedFactory.error(title, description).build();
    }

    public static MessageEmbed success(String title, String description) {
        return EmbedFactory.success(title, description).build();
    }

    public static MessageEmbed info(String title, String description) {
        return EmbedFactory.info(title, description).build();
    }

    public static MessageEmbed warning(String title, String description) {
        return EmbedFactory.warning(title, description).build();
    }

    public static MessageEmbed commandList(String prefix, java.util.Collection<kernel.loki.command.Command> commands) {
        EmbedBuilder builder = EmbedFactory.builder(EmbedColor.PRIMARY)
            .setTitle("Command List")
            .setDescription("Available commands (prefix: `" + prefix + "`)");

        for (kernel.loki.command.Command cmd : commands) {
            builder.addField(
                prefix + cmd.name(),
                cmd.description() + (cmd.usage().equals(cmd.name()) ? "" : "\nUsage: `" + prefix + cmd.usage() + "`"),
                false
            );
        }

        return builder.build();
    }

    public static MessageEmbed userInfo(User user, String extraInfo) {
        return EmbedFactory.userEmbed(user)
            .setDescription(extraInfo)
            .build();
    }
}
