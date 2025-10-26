use crate::{
    executor::Executor,
    metrics::{Collector, Registerer},
    parser::{volts::VoltsState, Parser},
};

#[derive(Clone, Debug)]
pub struct Volts<E, P, R> {
    executor: E,
    parser: P,
    registerer: R,
}

impl<E, P, R> Volts<E, P, R> {
    pub fn new(executor: E, parser: P, registerer: R) -> Self {
        Self {
            executor,
            parser,
            registerer,
        }
    }
}

impl<E, P, R> Collector for Volts<E, P, R>
where
    E: Executor<Output = Vec<String>> + Send + Sync,
    P: for<'a> Parser<Item<'a> = &'a str> + Send + Sync + 'static,
    R: Registerer<Item = VoltsState> + Send + Sync,
{
    fn name(&self) ->  &'static str {
        "volts"
    }

    #[tracing::instrument(skip_all, fields(collector = %std::any::type_name::<Self>()))]
    async fn collect(&self) -> anyhow::Result<()> {
        tracing::debug!("collecting volts");

        let outputs = self.executor.execute().await?;
        let outputs = outputs
            .iter()
            .map(|output| self.parser.parse(output)).collect::<anyhow::Result<Vec<_>>>()?;
        let state = VoltsState::try_from(outputs)?;

        self.registerer.register(state).await?;

        tracing::debug!("succeeded collecting volts");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::future::ok;

    use crate::{
        collector::volts::Volts,
        executor::Executor,
        metrics::{Collector, Registerer},
        parser::{volts::VoltsState, Parser},
    };

    mockall::mock! {
        Executor {}

        impl Executor for Executor {
            type Output = Vec<String>;

            fn execute(&self) -> impl Future<Output = anyhow::Result<<Self as Executor>::Output>> + Send;
        }
    }

    mockall::mock! {
        Registerer {}

        impl Registerer for Registerer {
            type Item = VoltsState;

            fn register(&self, state: <Self as Registerer>::Item) -> impl Future<Output = anyhow::Result<()>> + Send;
        }
    }

    mockall::mock! {
        Parser {}

        impl Parser for Parser {
            type Item<'a> = &'a str;

            fn parse<'a>(&self, input: &'a str) -> anyhow::Result<<Self as Parser>::Item<'static>>;
        }
    }

    #[tokio::test]
    async fn collect() {
        let mut mock_executor = MockExecutor::new();
        mock_executor
            .expect_execute()
            .times(1)
            .returning(|| Box::pin(ok(vec![
                "volt=1.3563V".to_string(),
                "volt=1.3564V".to_string(),
                "volt=1.3565V".to_string(),
                "volt=1.3566V".to_string(),
            ])));

        let mut mock_parser = MockParser::new();
        mock_parser
            .expect_parse()
            .times(1)
            .withf(|x| x == "volt=1.3563V")
            .returning(|_| Ok("1.3563"));

        mock_parser
            .expect_parse()
            .times(1)
            .withf(|x| x == "volt=1.3564V")
            .returning(|_| Ok("1.3564"));

        mock_parser
            .expect_parse()
            .times(1)
            .withf(|x| x == "volt=1.3565V")
            .returning(|_| Ok("1.3565"));

        mock_parser
            .expect_parse()
            .times(1)
            .withf(|x| x == "volt=1.3566V")
            .returning(|_| Ok("1.3566"));

        let mut mock_registerer = MockRegisterer::new();
        mock_registerer
            .expect_register()
            .times(1)
            .withf(|x| *x == VoltsState {
                core: 1.3563,
                sdram_c: 1.3564,
                sdram_i: 1.3565,
                sdram_p: 1.3566,
            })
            .returning(|_| Box::pin(ok(())));

        let volts = Volts::new(mock_executor, mock_parser, mock_registerer);
        let result = volts.collect().await;

        assert!(result.is_ok())
    }
}
