use std::fmt::Display;

use clap::{Args, Parser, ValueEnum};
use strum::Display as StrumDisplay;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Directive;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long, default_value_t = 8021)]
    pub port: u16,

    #[arg(long, value_enum, default_value_t = Log::Plain)]
    pub log_output: Log,

    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    #[command(flatten)]
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Args)]
pub struct Metrics {
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_values_t = [
            Metric::Throttled,
            Metric::Volts,
        ],
    )]
    pub enable_metrics: Vec<Metric>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Log {
    Plain,
    Json,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, ValueEnum, StrumDisplay, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Metric {
    Throttled,
    Volts,
}

impl Metrics {
    pub fn has_throttled(&self) -> bool {
        self.enable_metrics.contains(&Metric::Throttled)
    }

    pub fn has_volts(&self) -> bool {
        self.enable_metrics.contains(&Metric::Volts)
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.enable_metrics.iter().map(ToString::to_string).collect::<Vec<_>>().join(","))
    }
}

impl From<LogLevel> for Directive {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::TRACE.into(),
            LogLevel::Debug => LevelFilter::DEBUG.into(),
            LogLevel::Info => LevelFilter::INFO.into(),
            LogLevel::Warn => LevelFilter::WARN.into(),
            LogLevel::Error => LevelFilter::ERROR.into(),
        }
    }
}
