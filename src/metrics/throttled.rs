use std::sync::{Arc, Mutex};

use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};

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

pub struct ThrottledRegisterer {
    pub registry: Arc<Mutex<Registry>>,
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
    async fn collect(&self) -> anyhow::Result<()> {
        let output = self.executor.execute().await?;
        let state = self.parser.parse(&output)?;

        self.registerer.register(state).await?;

        Ok(())
    }
}

impl Registerer for ThrottledRegisterer {
    type Item = ThrottledState;

    async fn register(&self, state: Self::Item) -> anyhow::Result<()> {
        let family = Family::<ThrottledLabels, Gauge>::default();
        {
            self
                .registry
                .lock()
                .expect("failed to lock registry mutex")
                .register(
                    "raspi_throttled",
                    "Throttled state",
                    family.clone(),
                );
        }

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
                undervoltage_has_occured: true,
                arm_frequency_capping_has_occured: false,
                throttling_has_occured: true,
                soft_temperature_limit_has_occured: true,
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
                undervoltage_has_occured: true,
                arm_frequency_capping_has_occured: false,
                throttling_has_occured: true,
                soft_temperature_limit_has_occured: true,
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
                undervoltage_has_occured: true,
                arm_frequency_capping_has_occured: false,
                throttling_has_occured: true,
                soft_temperature_limit_has_occured: true,
            })
            .returning(|_| Box::pin(ok(())));

        let throttled = Throttled::new(mock_executor, mock_parser, mock_registerer);
        let result = throttled.collect().await;

        assert!(result.is_ok())
    }
}
