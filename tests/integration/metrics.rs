use std::sync::{Arc, Mutex};

use prometheus_client::registry::Registry;
use raspi_exporter::metrics::{
    throttled::{Throttled, ThrottledExecutor, ThrottledParser, ThrottledRegisterer},
    Handler,
    MetricsHandler,
};

#[tokio::test]
async fn metrics() {
    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = Throttled::new(
        ThrottledExecutor::new("echo", ["throttled=0xd0005"]),
        ThrottledParser,
        ThrottledRegisterer { registry: registry.clone() }
    );
    let metrics_handler = MetricsHandler::new(throttled, registry.clone());
    let result = metrics_handler.handle().await.unwrap();
    let mut lines = result.lines();

    assert_eq!(lines.clone().count(), 11);
    assert_eq!(lines.next(), Some("# HELP raspi_throttled Throttled state."));
    assert_eq!(lines.next(), Some("# TYPE raspi_throttled gauge"));

    let mut metrics = lines.by_ref().take(8).collect::<Vec<_>>();
    metrics.sort();

    assert_eq!(metrics.clone().len(), 8);
    assert_eq!(
        metrics,
        [
            "raspi_throttled{bit=\"0\"} 1",
            "raspi_throttled{bit=\"1\"} 0",
            "raspi_throttled{bit=\"16\"} 1",
            "raspi_throttled{bit=\"17\"} 0",
            "raspi_throttled{bit=\"18\"} 1",
            "raspi_throttled{bit=\"19\"} 1",
            "raspi_throttled{bit=\"2\"} 1",
            "raspi_throttled{bit=\"3\"} 0",
        ]
    );
    assert_eq!(lines.next(), Some("# EOF"))
}

#[tokio::test]
async fn command_not_found() {
    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = Throttled::new(
        ThrottledExecutor::new("command_not_found", []),
        ThrottledParser,
        ThrottledRegisterer { registry: registry.clone() }
    );
    let metrics_handler = MetricsHandler::new(throttled, registry.clone());
    let result = metrics_handler.handle().await;

    assert!(result.is_err());
}
