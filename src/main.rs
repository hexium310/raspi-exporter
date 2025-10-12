use std::sync::{Arc, Mutex};

use prometheus_client::registry::Registry;

use raspi_exporter::{
    metrics::{
        throttled::{Throttled, ThrottledRegisterer, ThrottledExecutor, ThrottledParser},
        MetricsHandler,
    },
    server::Server,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    setup_logging();

    tracing::info!("starting raspi_exporter");
    tracing::info!("enabled metrics: vcgencmd");

    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = Throttled::new(
        ThrottledExecutor::new("vcgencmd", ["get_throttled"]),
        ThrottledParser,
        ThrottledRegisterer { registry: registry.clone() }
    );
    let metrics_handler = MetricsHandler::new(throttled, registry.clone());

    let server = Server::new(8021, metrics_handler);
    if let Err(err) = server.start().await {
        tracing::error!("failed to start server\nError: {err:?}");
    };
}

fn setup_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
        )
        .init();
}
