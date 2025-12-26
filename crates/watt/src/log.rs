/// Imports
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;

/// Initializes logging
pub fn init() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::ENTER)
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .with_max_level(LevelFilter::TRACE)
        .pretty()
        .compact()
        .init();
}
