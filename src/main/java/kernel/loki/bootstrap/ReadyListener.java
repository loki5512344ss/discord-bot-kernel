package kernel.loki.bootstrap;

import com.google.inject.Singleton;
import kernel.loki.util.LoggerDecorator;
import net.dv8tion.jda.api.events.session.ReadyEvent;
import net.dv8tion.jda.api.hooks.ListenerAdapter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Singleton
public class ReadyListener extends ListenerAdapter {
    private static final Logger logger = LoggerFactory.getLogger(ReadyListener.class);

    @Override
    public void onReady(ReadyEvent event) {
        var selfUser = event.getJDA().getSelfUser();
        LoggerDecorator.bot(logger, "Logged in as: {}#{}", selfUser.getName(), selfUser.getDiscriminator());
        LoggerDecorator.bot(logger, "Connected to {} guilds", event.getGuildTotalCount());
    }
}
