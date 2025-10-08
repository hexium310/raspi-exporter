use std::ffi::OsStr;

use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

use crate::{command::{Executor, Parser}, metrics::Collector};

#[derive(Clone, Debug)]
pub struct Throttled<S, I, P> {
    command: S,
    args: I,
    parser: P,
}

// https://www.raspberrypi.com/documentation/computers/os.html#get_throttled
pub struct ThrottledState {
    pub undervoltage_detected: bool,
    pub arm_frequency_capped: bool,
    pub currently_throttled: bool,
    pub soft_temperature_limit_active: bool,
    pub undervoltage_has_occured: bool,
    pub arm_frequency_capping_has_occured: bool,
    pub throttling_has_occured: bool,
    pub soft_temperature_limit_has_occured: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ThrottledLabels {
    bit: u8
}

pub struct ThrottledParser;

impl<S, I, P> Throttled<S, I, P> {
    pub fn new(command: S, args: I, parser: P) -> Self {
        Self {
            command,
            args,
            parser,
        }
    }
}

impl<S, I, P> Executor<S, I> for Throttled<S, I, P>
where 
    S: AsRef<OsStr> + Send,
    I: IntoIterator<Item = S> + Send,
{}

impl<S, I, P> Collector for Throttled<S, I, P>
where 
    S: AsRef<OsStr> + Clone + Copy + Send + Sync,
    I: IntoIterator<Item = S> + Clone + Copy + Send + Sync,
    P: Parser<Item = ThrottledState> + Send + Sync,
{
    async fn collect(&self, registry: &mut Registry) -> anyhow::Result<()> {
        let family = Family::<ThrottledLabels, Gauge>::default();
        registry.register(
            "raspi_throttled",
            "Throttled state",
            family.clone(),
        );

        let output = self.execute(self.command, self.args).await?;
        let state = self.parser.parse(&output)?;

        family.get_or_create(&ThrottledLabels { bit: 0 }).set(state.undervoltage_detected.into());
        family.get_or_create(&ThrottledLabels { bit: 1 }).set(state.arm_frequency_capped.into());
        family.get_or_create(&ThrottledLabels { bit: 2 }).set(state.currently_throttled.into());
        family.get_or_create(&ThrottledLabels { bit: 3 }).set(state.soft_temperature_limit_active.into());
        family.get_or_create(&ThrottledLabels { bit: 16 }).set(state.undervoltage_has_occured.into());
        family.get_or_create(&ThrottledLabels { bit: 17 }).set(state.arm_frequency_capping_has_occured.into());
        family.get_or_create(&ThrottledLabels { bit: 18 }).set(state.throttling_has_occured.into());
        family.get_or_create(&ThrottledLabels { bit: 19 }).set(state.soft_temperature_limit_has_occured.into());

        Ok(())
    }
}

impl Parser for ThrottledParser {
    type Item = ThrottledState;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        let hex = match input.trim().split_once('=') {
            Some((_key, value)) => value,
            None => anyhow::bail!("failed to parse: {input}"),
        };
        let decimal = u32::from_str_radix(&hex[2..], 16)?;

        let state = Self::Item {
            undervoltage_detected: decimal & 0b1 << 0 != 0,
            arm_frequency_capped: decimal & 0b1 << 1 != 0,
            currently_throttled: decimal & 0b1 << 2 != 0,
            soft_temperature_limit_active: decimal & 0b1 << 3 != 0,
            undervoltage_has_occured: decimal & 0b1 << 16 != 0,
            arm_frequency_capping_has_occured: decimal & 0b1 << 17 != 0,
            throttling_has_occured: decimal & 0b1 << 18 != 0,
            soft_temperature_limit_has_occured: decimal & 0b1 << 19 != 0,
        };

        Ok(state)
    }
}
