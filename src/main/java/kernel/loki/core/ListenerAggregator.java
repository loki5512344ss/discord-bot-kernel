package kernel.loki.core;

import com.google.inject.Inject;
import com.google.inject.Singleton;
import java.util.ArrayList;
import java.util.List;
import kernel.loki.bootstrap.ReadyListener;

@Singleton
public class ListenerAggregator {
    private final List<Object> listeners = new ArrayList<>();

    @Inject
    public ListenerAggregator(
        ReadyListener readyListener,
        MessageListener messageListener
    ) {
        listeners.add(readyListener);
        listeners.add(messageListener);
    }

    public Object[] all() {
        return listeners.toArray();
    }

    public void add(Object listener) {
        listeners.add(listener);
    }
}
