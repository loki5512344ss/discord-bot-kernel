package kernel.loki.feature.info;

import com.google.inject.Inject;
import kernel.loki.core.registry.CommandRegistrar;
import kernel.loki.core.registry.CommandRegistry;
import kernel.loki.feature.info.command.AvatarCommand;
import kernel.loki.feature.info.command.PingCommand;
import kernel.loki.feature.info.command.ServerInfoCommand;

public class InfoCommandRegistrar implements CommandRegistrar {
    private final PingCommand ping;
    private final AvatarCommand avatar;
    private final ServerInfoCommand serverInfo;

    @Inject
    public InfoCommandRegistrar(PingCommand ping, AvatarCommand avatar, ServerInfoCommand serverInfo) {
        this.ping = ping;
        this.avatar = avatar;
        this.serverInfo = serverInfo;
    }

    @Override
    public void register(CommandRegistry registry) {
        registry.register(ping);
        registry.register(avatar);
        registry.register(serverInfo);
    }
}
