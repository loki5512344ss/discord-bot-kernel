package kernel.loki.core.bootstrap;

public enum BootstrapPhase {
    CONFIG_LOADING,
    DATABASE_CONNECT,
    DATABASE_MIGRATE,
    DI_INITIALIZE,
    REGISTRATION,
    LIFECYCLE_START,
    JDA_BUILD,
    JDA_READY,
    COMPLETE
}
