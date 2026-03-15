use camino::Utf8PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use watt_orchestra::{Orchestrator, OrchestratorConfig};

fn main() {
    let filter: EnvFilter = EnvFilter::builder()
        .with_env_var("WATT_LOG")
        .with_default_directive(LevelFilter::OFF.into())
        .from_env_lossy();

    let fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::ENTER)
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    let mut orchestrator = Orchestrator::new(OrchestratorConfig::new(
        Utf8PathBuf::from("/home/vyacheslav/watt/test/src"),
        Utf8PathBuf::from("/home/vyacheslav/watt/test/outcome"),
    ));
    orchestrator.perform_compilation();
}
