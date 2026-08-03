package kernel.loki.core;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import net.dv8tion.jda.api.hooks.ListenerAdapter;

@Singleton
public class MessageListener extends ListenerAdapter {
    private final EventDispatcher dispatcher;

    @Inject
    public MessageListener(EventDispatcher dispatcher) {
        this.dispatcher = dispatcher;
    }

    @Override
    public void onMessageReceived(MessageReceivedEvent event) {
        dispatcher.dispatch(event);
    }
}
