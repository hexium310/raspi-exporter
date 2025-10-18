use std::sync::{Arc, Mutex};

use clap::Parser;
use prometheus_client::registry::Registry;

use raspi_exporter::{
    cli::{ Cli, Log },
    collector::throttled::Throttled,
    executor::throttled::ThrottledExecutor,
    metrics::MetricsHandler,
    parser::throttled::ThrottledParser,
    registerer::throttled::ThrottledRegisterer,
    server::Server,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    setup_logging(args.log);

    tracing::info!("starting raspi_exporter");
    tracing::info!("enabled metrics: {}", args.metrics);

    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = args
        .metrics
        .has_throttled()
        .then(|| Throttled::new(
            ThrottledExecutor::new("vcgencmd", ["get_throttled"]),
            ThrottledParser,
            ThrottledRegisterer { registry: registry.clone() }
        ));
    let metrics_handler = MetricsHandler::new(throttled, registry.clone());

    let server = Server::new(args.port, metrics_handler);
    if let Err(err) = server.start().await {
        tracing::error!("failed to start server\nError: {err:?}");
    };
}

fn setup_logging(output_type: Log) {
    let layer = fmt::layer();
    let layer = match output_type {
        Log::Plain => layer.boxed(),
        Log::Json => layer.json().boxed(),
    };

    tracing_subscriber::registry()
        .with(layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
        )
        .init();
}
