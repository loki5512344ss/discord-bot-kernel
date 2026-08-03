package kernel.loki.feature.info.command;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import java.time.format.DateTimeFormatter;
import kernel.loki.command.Command;
import kernel.loki.command.CommandCategory;
import kernel.loki.command.CommandContext;
import kernel.loki.embed.EmbedFactory;
import kernel.loki.embed.EmbedColor;
import net.dv8tion.jda.api.entities.Guild;

@Singleton
public class ServerInfoCommand implements Command {
    @Inject
    public ServerInfoCommand() {}

    @Override
    public String name() { return "serverinfo"; }

    @Override
    public String description() { return "Show server information"; }

    @Override
    public String category() { return CommandCategory.INFO.display(); }

    @Override
    public void execute(CommandContext ctx) {
        Guild guild = ctx.guild();
        ctx.channel().sendMessageEmbeds(
            EmbedFactory.builder(EmbedColor.PRIMARY)
                .setTitle(guild.getName())
                .setThumbnail(guild.getIconUrl())
                .addField("Owner", guild.getOwner() != null
                    ? guild.getOwner().getEffectiveName() : "Unknown", true)
                .addField("Members", String.valueOf(guild.getMemberCount()), true)
                .addField("Boost Tier", guild.getBoostTier().name(), true)
                .addField("Boost Count", String.valueOf(guild.getBoostCount()), true)
                .addField("Channels", String.valueOf(guild.getChannels().size()), true)
                .addField("Roles", String.valueOf(guild.getRoles().size()), true)
                .addField("Created",
                    guild.getTimeCreated().format(DateTimeFormatter.RFC_1123_DATE_TIME), false)
                .setFooter("ID: " + guild.getId())
                .build())
            .queue();
    }
}
