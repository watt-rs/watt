/// Imports
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Initializes logging
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off"));
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
