package kernel.loki.command;

import java.util.List;
import net.dv8tion.jda.api.entities.Guild;
import net.dv8tion.jda.api.entities.Member;
import net.dv8tion.jda.api.entities.Message;
import net.dv8tion.jda.api.entities.User;
import net.dv8tion.jda.api.entities.channel.middleman.MessageChannel;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;

public record CommandContext(
    Message message,
    List<String> args,
    MessageChannel channel,
    Member member,
    Guild guild,
    User author,
    MessageReceivedEvent event
) {
    public CommandContext(MessageReceivedEvent event, List<String> args) {
        this(
            event.getMessage(),
            args,
            event.getChannel(),
            event.getMember(),
            event.getGuild(),
            event.getAuthor(),
            event
        );
    }
}
