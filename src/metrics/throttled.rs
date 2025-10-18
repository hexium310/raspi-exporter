use std::sync::{Arc, Mutex};

use anyhow::Context;
use prometheus_client::{
    encoding::{EncodeLabelSet, EncodeLabelValue, LabelValueEncoder},
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use strum::Display as StrumDisplay;

use crate::{command::{CommandExecutor, Executor, Parser}, metrics::{Collector, Registerer}};

pub type ThrottledExecutor<S, I> = CommandExecutor<S, I>;

#[derive(Clone, Debug)]
pub struct Throttled<E, P, R> {
    executor: E,
    parser: P,
    registerer: R,
}

// https://www.raspberrypi.com/documentation/computers/os.html#get_throttled
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ThrottledState {
    pub undervoltage_detected: bool,
    pub arm_frequency_capped: bool,
    pub currently_throttled: bool,
    pub soft_temperature_limit_active: bool,
    pub undervoltage_has_occurred: bool,
    pub arm_frequency_capping_has_occurred: bool,
    pub throttling_has_occurred: bool,
    pub soft_temperature_limit_has_occurred: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ThrottlingActiveLabels {
    kind: ThrottlingKind,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ThrottlingOccurredLabels {
    kind: ThrottlingKind,
}

#[derive(Debug)]
pub struct ThrottledParser;

#[derive(Debug)]
pub struct ThrottledRegisterer {
    pub registry: Arc<Mutex<Registry>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, StrumDisplay)]
pub enum ThrottlingKind {
    #[strum(to_string = "undervoltage")]
    Undervoltage,
    #[strum(to_string = "arm frequency")]
    ArmFrequency,
    #[strum(to_string = "throttled")]
    Throttled,
    #[strum(to_string = "soft temperature limit")]
    SoftTemperatureLimit,
}

impl<E, P, R> Throttled<E, P, R> {
    pub fn new(executor: E, parser: P, registerer: R) -> Self {
        Self {
            executor,
            parser,
            registerer,
        }
    }
}

impl<E, P, R> Collector for Throttled<E, P, R>
where
    E: Executor + Send + Sync,
    P: Parser<Item = ThrottledState> + Send + Sync,
    R: Registerer<Item = ThrottledState> + Send + Sync,
{
    fn name(&self) ->  &'static str {
        "throttled"
    }

    #[tracing::instrument(skip_all, fields(collector = %std::any::type_name::<Self>()))]
    async fn collect(&self) -> anyhow::Result<()> {
        tracing::debug!("collecting throttled");

        let output = self.executor.execute().await?;
        let state = self.parser.parse(&output)?;

        self.registerer.register(state).await?;

        tracing::debug!("succeeded collecting throttled");

        Ok(())
    }
}

impl Registerer for ThrottledRegisterer {
    type Item = ThrottledState;

    async fn register(&self, state: Self::Item) -> anyhow::Result<()> {
        // Substitutes Gauge for StateSet of OpenMetrics because prometheus_client doens't implement it
        let throttling_active_family = Family::<ThrottlingActiveLabels, Gauge>::default();
        // Substitutes Gauge for StateSet of OpenMetrics because prometheus_client doens't implement it
        let throttling_occurred_family = Family::<ThrottlingOccurredLabels, Gauge>::default();
        {
            let mut registry = self.registry.lock().expect("failed to lock registry mutex");
            registry.register(
                "raspi_throttling_active",
                "State about throttling active currently",
                throttling_active_family.clone(),
            );
            registry.register(
                "raspi_throttling_occurred",
                "State about throttling occurred in the past",
                throttling_occurred_family.clone(),
            );
        }

        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::Undervoltage }).set(state.undervoltage_detected.into());
        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::ArmFrequency }).set(state.arm_frequency_capped.into());
        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::Throttled }).set(state.currently_throttled.into());
        throttling_active_family.get_or_create(&ThrottlingActiveLabels { kind: ThrottlingKind::SoftTemperatureLimit }).set(state.soft_temperature_limit_active.into());

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::Undervoltage });
            if state.undervoltage_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::ArmFrequency });
            if state.arm_frequency_capping_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::Throttled });
            if state.throttling_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        {
            let metric = throttling_occurred_family.get_or_create(&ThrottlingOccurredLabels { kind: ThrottlingKind::SoftTemperatureLimit });
            if state.soft_temperature_limit_has_occurred && metric.get() == 0 {
                metric.inc();
            }
        }

        Ok(())
    }
}

impl Parser for ThrottledParser {
    type Item = ThrottledState;

    fn parse(&self, input: &str) -> anyhow::Result<Self::Item> {
        let invalid_input_error = || format!("invalid input: {input}");

        let decimal = input
            .trim()
            .split_once('=')
            .with_context(invalid_input_error)
            .and_then(|(_, v)| u32::from_str_radix(&v[2..], 16).map_err(|_| anyhow::anyhow!(invalid_input_error())))?;

        let state = Self::Item {
            undervoltage_detected: decimal & 0b1 << 0 != 0,
            arm_frequency_capped: decimal & 0b1 << 1 != 0,
            currently_throttled: decimal & 0b1 << 2 != 0,
            soft_temperature_limit_active: decimal & 0b1 << 3 != 0,
            undervoltage_has_occurred: decimal & 0b1 << 16 != 0,
            arm_frequency_capping_has_occurred: decimal & 0b1 << 17 != 0,
            throttling_has_occurred: decimal & 0b1 << 18 != 0,
            soft_temperature_limit_has_occurred: decimal & 0b1 << 19 != 0,
        };

        Ok(state)
    }
}

impl EncodeLabelValue for ThrottlingKind {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), std::fmt::Error> {
        self.to_string().encode(encoder)
    }
}

#[cfg(test)]
mod tests {
    use futures::future::ok;

    use crate::{command::{MockExecutor, Parser}, metrics::{throttled::{Throttled, ThrottledParser, ThrottledState}, Collector, Registerer}};

    mockall::mock! {
        Registerer {}

        impl Registerer for Registerer {
            type Item = ThrottledState;

            fn register(&self, state: <Self as Registerer>::Item) -> impl Future<Output = anyhow::Result<()>> + Send;
        }
    }

    mockall::mock! {
        Parser {}

        impl Parser for Parser {
            type Item = ThrottledState;

            fn parse(&self, input: &str) -> anyhow::Result<<Self as Parser>::Item>;
        }
    }

    #[test]
    fn parse() {
        let throttled_parser = ThrottledParser;
        let result = throttled_parser.parse("throttled=0xd0005").unwrap();

        assert_eq!(
            result,
            ThrottledState {
                undervoltage_detected: true,
                arm_frequency_capped: false,
                currently_throttled: true,
                soft_temperature_limit_active: false,
                undervoltage_has_occurred: true,
                arm_frequency_capping_has_occurred: false,
                throttling_has_occurred: true,
                soft_temperature_limit_has_occurred: true,
            }
        )
    }

    #[tokio::test]
    async fn collect() {
        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_execute()
            .times(1)
            .returning(|| Box::pin(ok("throttled=0xd0005".to_string())));

        let mut mock_parser = MockParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .withf(|x| x == "throttled=0xd0005")
            .returning(|_| Ok(ThrottledState {
                undervoltage_detected: true,
                arm_frequency_capped: false,
                currently_throttled: true,
                soft_temperature_limit_active: false,
                undervoltage_has_occurred: true,
                arm_frequency_capping_has_occurred: false,
                throttling_has_occurred: true,
                soft_temperature_limit_has_occurred: true,
            }));

        let mut mock_registerer = MockRegisterer::new();
        mock_registerer
            .expect_register()
            .times(1)
            .withf(|x| *x == ThrottledState {
                undervoltage_detected: true,
                arm_frequency_capped: false,
                currently_throttled: true,
                soft_temperature_limit_active: false,
                undervoltage_has_occurred: true,
                arm_frequency_capping_has_occurred: false,
                throttling_has_occurred: true,
                soft_temperature_limit_has_occurred: true,
            })
            .returning(|_| Box::pin(ok(())));

        let throttled = Throttled::new(mock_executor, mock_parser, mock_registerer);
        let result = throttled.collect().await;

        assert!(result.is_ok())
    }
}
