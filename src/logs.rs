use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
    prelude::*,
};

pub fn setup_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}
