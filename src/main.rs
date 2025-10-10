use std::sync::{Arc, Mutex};

use prometheus_client::registry::Registry;

use raspi_exporter::{
    metrics::{
        throttled::{Throttled, ThrottledRegisterer, ThrottledExecutor, ThrottledParser},
        MetricsHandler,
    },
    server::Server,
};

#[tokio::main]
async fn main() {
    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = Throttled::new(
        ThrottledExecutor::new("vcgencmd", ["get_throttled"]),
        ThrottledParser,
        ThrottledRegisterer { registry: registry.clone() }
    );
    let metrics_handler = MetricsHandler::new(throttled, registry.clone());

    let server = Server::new(8021, metrics_handler);
    server.start().await.unwrap();
}
