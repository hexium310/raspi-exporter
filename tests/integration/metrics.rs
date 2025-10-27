use std::sync::{Arc, Mutex};

use prometheus_client::registry::Registry;
use raspi_exporter::{
    collector::{throttled::Throttled, volts::Volts},
    executor::{throttled::ThrottledExecutor, volts::VoltsExecutor},
    metrics::{ Handler, MetricsHandler },
    parser::{throttled::ThrottledParser, volts::VoltsParser},
    registerer::{throttled::ThrottledRegisterer, volts::VoltsRegisterer},
};

#[tokio::test]
async fn metrics() {
    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = Throttled::new(
        ThrottledExecutor::new("echo", ["throttled=0xd0005"]),
        ThrottledParser,
        ThrottledRegisterer { registry: registry.clone() }
    );
    let volts = Volts::new(
        VoltsExecutor::new(&[
            ("echo", ["volt=1.3563V"]),
            ("echo", ["volt=1.3564V"]),
            ("echo", ["volt=1.3565V"]),
            ("echo", ["volt=1.3566V"]),
        ]),
        VoltsParser,
        VoltsRegisterer { registry: registry.clone() }
    );
    let metrics_handler = MetricsHandler::new(Some(throttled), Some(volts), registry.clone());
    let result = metrics_handler.handle().await.unwrap();
    let mut lines = result.lines();

    assert_eq!(lines.clone().count(), 19);
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

    assert_eq!(lines.next(), Some("# HELP raspi_volts Current voltage."));
    assert_eq!(lines.next(), Some("# TYPE raspi_volts gauge"));

    let mut metrics = lines.by_ref().take(4).collect::<Vec<_>>();
    metrics.sort();
    assert_eq!(metrics.clone().len(), 4);
    assert_eq!(
        metrics,
        [
            "raspi_volts{kind=\"SDRAM I/O\"} 1.3565",
            "raspi_volts{kind=\"SDRAM PHY\"} 1.3566",
            "raspi_volts{kind=\"SDRAM core\"} 1.3564",
            "raspi_volts{kind=\"VC4 core\"} 1.3563",
        ]
    );

    assert_eq!(lines.next(), Some("# EOF"));
    assert_eq!(lines.next(), None);
}

#[tokio::test]
async fn command_not_found() {
    let registry = Arc::new(Mutex::new(Registry::default()));
    let throttled = Throttled::new(
        ThrottledExecutor::new("command_not_found", []),
        ThrottledParser,
        ThrottledRegisterer { registry: registry.clone() }
    );
    let volts = Volts::new(
        VoltsExecutor::new(&[
            ("command_not_found", ["measure_volts", "core"]),
            ("command_not_found", ["measure_volts", "sdram_c"]),
            ("command_not_found", ["measure_volts", "sdram_i"]),
            ("command_not_found", ["measure_volts", "sdram_p"]),
        ]),
        VoltsParser,
        VoltsRegisterer { registry: registry.clone() }
    );
    let metrics_handler = MetricsHandler::new(Some(throttled), Some(volts), registry.clone());
    let result = metrics_handler.handle().await.unwrap();
    let mut lines = result.lines();

    assert_eq!(lines.clone().count(), 1);
    assert_eq!(lines.next(), Some("# EOF"));
}
