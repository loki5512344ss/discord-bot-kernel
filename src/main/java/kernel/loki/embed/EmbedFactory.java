package kernel.loki.embed;

import java.awt.Color;
import java.time.Instant;
import net.dv8tion.jda.api.EmbedBuilder;
import net.dv8tion.jda.api.entities.MessageEmbed;
import net.dv8tion.jda.api.entities.User;

public class EmbedFactory {
    private EmbedFactory() {}

    public static EmbedBuilder builder() {
        return new EmbedBuilder()
            .setTimestamp(Instant.now());
    }

    public static EmbedBuilder builder(Color color) {
        return builder().setColor(color);
    }

    public static EmbedBuilder primary(String title, String description) {
        return builder(EmbedColor.PRIMARY)
            .setTitle(title)
            .setDescription(description);
    }

    public static EmbedBuilder success(String title, String description) {
        return builder(EmbedColor.SUCCESS)
            .setTitle(title)
            .setDescription(description);
    }

    public static EmbedBuilder warning(String title, String description) {
        return builder(EmbedColor.WARNING)
            .setTitle(title)
            .setDescription(description);
    }

    public static EmbedBuilder error(String title, String description) {
        return builder(EmbedColor.ERROR)
            .setTitle(title)
            .setDescription(description);
    }

    public static EmbedBuilder info(String title, String description) {
        return builder(EmbedColor.INFO)
            .setTitle(title)
            .setDescription(description);
    }

    public static EmbedBuilder userEmbed(User user) {
        return builder(EmbedColor.PRIMARY)
            .setAuthor(user.getName(), null, user.getAvatarUrl())
            .setThumbnail(user.getAvatarUrl());
    }

    public static MessageEmbed footer(EmbedBuilder builder, String text, String iconUrl) {
        return builder
            .setFooter(text, iconUrl)
            .build();
    }
}
