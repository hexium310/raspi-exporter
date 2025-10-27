use std::sync::{Arc, Mutex};

use clap::Parser;
use prometheus_client::registry::Registry;

use raspi_exporter::{
    cli::{ Cli, Log, LogLevel },
    collector::{throttled::Throttled, volts::Volts},
    executor::{throttled::ThrottledExecutor, volts::VoltsExecutor},
    metrics::MetricsHandler,
    parser::{throttled::ThrottledParser, volts::VoltsParser},
    registerer::{throttled::ThrottledRegisterer, volts::VoltsRegisterer},
    server::Server,
};
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

    setup_logging(args.log_level, args.log_output);

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
    let volts = args
        .metrics
        .has_volts()
        .then(|| Volts::new(
            VoltsExecutor::new(&[
                ("vcgencmd", ["measure_volts", "core"]),
                ("vcgencmd", ["measure_volts", "sdram_c"]),
                ("vcgencmd", ["measure_volts", "sdram_i"]),
                ("vcgencmd", ["measure_volts", "sdram_p"]),
            ]),
            VoltsParser,
            VoltsRegisterer { registry: registry.clone() }
        ));
    let metrics_handler = MetricsHandler::new(throttled, volts, registry.clone());

    let server = Server::new(args.port, metrics_handler);
    if let Err(err) = server.start().await {
        tracing::error!("failed to start server\nError: {err:?}");
    };
}

fn setup_logging(level: LogLevel, output_type: Log) {
    let layer = fmt::layer();
    let layer = match output_type {
        Log::Plain => layer.boxed(),
        Log::Json => layer.json().boxed(),
    };

    tracing_subscriber::registry()
        .with(layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy()
        )
        .init();
}
