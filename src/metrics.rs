use std::sync::{Arc, Mutex};

use prometheus_client::{encoding::text, registry::Registry};

pub mod throttled;

pub struct MetricsHandler<Throttled> {
    throttled: Throttled,
    registry: Arc<Mutex<Registry>>,
}

pub trait Registerer {
    type Item;

    fn register(&self, state: Self::Item) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait Collector {
    fn collect(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait Handler {
    fn handle(&self) -> impl Future<Output = anyhow::Result<String>> + Send;
}

impl<Throttled> MetricsHandler<Throttled> {
    pub fn new(throttled: Throttled, registry: Arc<Mutex<Registry>>) -> Self {
        Self {
            throttled,
            registry,
        }
    }
}

impl<Throttled> Handler for MetricsHandler<Throttled>
where
    Throttled: Collector + Send + Sync + 'static,
{
    async fn handle(&self) -> anyhow::Result<String> {
        self.throttled.collect().await?;

        let mut buffer = String::new();
        {
            text::encode(&mut buffer, &self.registry.lock().expect("failed to lock registry mutex"))?;
        }

        Ok(buffer)
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

        let metrics_handler = MetricsHandler::new(mock_throttled, Arc::new(Mutex::new(Registry::default())));
        let result = metrics_handler.handle().await.unwrap();

        assert_eq!(result, "# EOF\n")
    }
}
