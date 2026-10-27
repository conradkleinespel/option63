use tracing_subscriber::EnvFilter;

pub fn default_filter(verbose: bool) -> &'static str {
    if verbose { "debug" } else { "info" }
}

pub fn init(verbose: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter(verbose)));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
