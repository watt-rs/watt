use tracing::level_filters::LevelFilter;
/// Imports
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Initializes logging
pub fn init() {
    let filter: EnvFilter = EnvFilter::builder()
        .with_env_var("WATT_LOG")
        .with_default_directive(LevelFilter::OFF.into())
        .from_env_lossy();

    let fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::ENTER)
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .pretty()
        .compact();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
