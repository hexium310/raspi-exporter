use crate::{
    executor::Executor,
    metrics::{Collector, Registerer},
    parser::{throttled::ThrottledState, Parser},
};

#[derive(Clone, Debug)]
pub struct Throttled<E, P, R> {
    executor: E,
    parser: P,
    registerer: R,
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

#[cfg(test)]
mod tests {
    use futures::future::ok;

    use crate::{
        collector::throttled::Throttled,
        executor::MockExecutor,
        metrics::{Collector, Registerer},
        parser::{throttled::ThrottledState, Parser},
    };

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
