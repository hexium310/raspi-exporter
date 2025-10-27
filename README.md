# raspi-exporter

raspi-exporter exports Raspberry Pi metrics for Prometheus.

## Available collectors

- throttled (by `vcgencmd get_throttled`)
- volts (by `vcgencmd measure_volts [block]`)

## Installation

Binaries can be downloaded from [releases][].

## Usage

```sh
raspi-exporter --port 8021 --enable-metrics throttled,volts
```

See `raspi-exporter --help` for more usage.

[releases]: [https://github.com/hexium310/raspi-exporter/releases]
