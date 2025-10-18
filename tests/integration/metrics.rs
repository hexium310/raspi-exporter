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
    let metrics_handler = MetricsHandler::new(Some(throttled), registry.clone());
    let result = metrics_handler.handle().await.unwrap();
    let mut lines = result.lines();

    assert_eq!(lines.clone().count(), 13);
    assert_eq!(lines.next(), Some("# HELP raspi_throttling_active State about throttling active currently."));
    assert_eq!(lines.next(), Some("# TYPE raspi_throttling_active gauge"));

    let mut metrics = lines.by_ref().take(4).collect::<Vec<_>>();
    metrics.sort();

    assert_eq!(metrics.clone().len(), 4);
    assert_eq!(
        metrics,
        [
            "raspi_throttling_active{kind=\"arm frequency\"} 0",
            "raspi_throttling_active{kind=\"soft temperature limit\"} 0",
            "raspi_throttling_active{kind=\"throttled\"} 1",
            "raspi_throttling_active{kind=\"undervoltage\"} 1",
        ]
    );

    assert_eq!(lines.next(), Some("# HELP raspi_throttling_occurred State about throttling occurred in the past."));
    assert_eq!(lines.next(), Some("# TYPE raspi_throttling_occurred gauge"));

    let mut metrics = lines.by_ref().take(4).collect::<Vec<_>>();
    metrics.sort();
    assert_eq!(metrics.clone().len(), 4);
    assert_eq!(
        metrics,
        [
            "raspi_throttling_occurred{kind=\"arm frequency\"} 0",
            "raspi_throttling_occurred{kind=\"soft temperature limit\"} 1",
            "raspi_throttling_occurred{kind=\"throttled\"} 1",
            "raspi_throttling_occurred{kind=\"undervoltage\"} 1",
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
    let metrics_handler = MetricsHandler::new(Some(throttled), registry.clone());
    let result = metrics_handler.handle().await.unwrap();
    let mut lines = result.lines();

    assert_eq!(lines.clone().count(), 1);
    assert_eq!(lines.next(), Some("# EOF"));
}
