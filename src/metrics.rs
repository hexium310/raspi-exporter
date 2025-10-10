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
