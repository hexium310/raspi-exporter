use std::fmt::Display;

use clap::{Args, Parser, ValueEnum};
use strum::Display as StrumDisplay;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long, default_value_t = 8021)]
    pub port: u16,

    #[arg(long, value_enum, default_value_t = Log::Plain)]
    pub log: Log,

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
        ],
    )]
    pub enable_metrics: Vec<Metric>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Log {
    Plain,
    Json,
}

#[derive(Debug, Clone, ValueEnum, StrumDisplay, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Metric {
    Throttled,
}

impl Metrics {
    pub fn has_throttled(&self) -> bool {
        self.enable_metrics.contains(&Metric::Throttled)
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.enable_metrics.iter().map(ToString::to_string).collect::<Vec<_>>().join(","))
    }
}
