/// Imports
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

/// Initializes logging
pub fn init() {
    if let Ok(log) = std::env::var("RUST_LOG") {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(&format!("watt={log}")))
            .with_span_events(FmtSpan::ENTER)
            .with_target(false)
            .with_level(true)
            .with_line_number(true)
            .pretty()
            .compact()
            .init();
    }
}
