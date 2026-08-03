package kernel.loki.feature.info.command;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import kernel.loki.command.Command;
import kernel.loki.command.CommandCategory;
import kernel.loki.command.CommandContext;
import kernel.loki.embed.EmbedFactory;
import kernel.loki.embed.EmbedColor;
import net.dv8tion.jda.api.entities.Member;

@Singleton
public class AvatarCommand implements Command {
    @Inject
    public AvatarCommand() {}

    @Override
    public String name() { return "avatar"; }

    @Override
    public String description() { return "Get a user's avatar"; }

    @Override
    public String usage() { return "avatar [@user]"; }

    @Override
    public String category() { return CommandCategory.INFO.display(); }

    @Override
    public void execute(CommandContext ctx) {
        Member target = ctx.args().isEmpty()
            ? ctx.member()
            : ctx.message().getMentions().getMembers().stream().findFirst().orElse(ctx.member());

        ctx.channel().sendMessageEmbeds(
            EmbedFactory.builder(EmbedColor.PRIMARY)
                .setTitle(target.getEffectiveName() + "'s Avatar")
                .setImage(target.getEffectiveAvatarUrl() + "?size=1024")
                .setFooter("ID: " + target.getId())
                .build())
            .queue();
    }
}
