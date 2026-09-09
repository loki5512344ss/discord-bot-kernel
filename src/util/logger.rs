pub fn init() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,serenity=warn".to_owned());
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::new(filter)).init();
}
