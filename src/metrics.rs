use std::sync::{Arc, Mutex};

use anyhow::Context;
use prometheus_client::{encoding::text, registry::Registry};

pub mod throttled;
pub mod volts;

#[derive(Debug)]
pub struct MetricsHandler<ThrottledCollector, VoltsCollector> {
    throttled: Option<ThrottledCollector>,
    volts: Option<VoltsCollector>,
    registry: Arc<Mutex<Registry>>,
}

pub trait Registerer {
    type Item;

    fn register(&self, state: Self::Item) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait Collector {
    fn name(&self) -> &'static str;
    fn collect(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait Handler {
    fn handle(&self) -> impl Future<Output = anyhow::Result<String>> + Send;
}

impl<ThrottledCollector, VoltsCollector> MetricsHandler<ThrottledCollector, VoltsCollector> {
    pub fn new(
        throttled: Option<ThrottledCollector>,
        volts: Option<VoltsCollector>,
        registry: Arc<Mutex<Registry>>,
    ) -> Self {
        Self {
            throttled,
            volts,
            registry,
        }
    }
}

impl<ThrottledCollector, VoltsCollector> Handler for MetricsHandler<ThrottledCollector, VoltsCollector>
where
    ThrottledCollector: Collector + Send + Sync,
    VoltsCollector: Collector + Send + Sync,
{
    #[tracing::instrument(skip_all)]
    async fn handle(&self) -> anyhow::Result<String> {
        collect(&self.throttled).await;
        collect(&self.volts).await;

        let mut buffer = String::new();
        tracing::debug!("encoding metrics");
        text::encode(&mut buffer, &self.registry.lock().expect("failed to lock registry mutex"))?;

        Ok(buffer)
    }
}

async fn collect<C: Collector + Send + Sync>(collector: &Option<C>) {
    if let Some(collector) = collector
        && let Err(err) = collector.collect().await.with_context(|| format!("{} collector error", collector.name()))
    {
        tracing::error!("{err:?}");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use futures::future::ok;
    use prometheus_client::registry::Registry;

    use crate::metrics::{
        Handler,
        MetricsHandler,
        MockCollector,
    };

    #[tokio::test]
    async fn handle() {
        let mut mock_throttled = MockCollector::new();
        mock_throttled
            .expect_collect()
            .times(1)
            .returning(|| Box::pin(ok(())));

        let mut mock_volts = MockCollector::new();
        mock_volts
            .expect_collect()
            .times(1)
            .returning(|| Box::pin(ok(())));

        let metrics_handler = MetricsHandler::new(
            Some(mock_throttled),
            Some(mock_volts),
            Arc::new(Mutex::new(Registry::default())),
        );
        let result = metrics_handler.handle().await.unwrap();

        assert_eq!(result, "# EOF\n")
    }
}
