use prometheus_client::{encoding::text, registry::Registry};

pub mod throttled;

pub struct MetricsHandler<Throttled> {
    throttled: Throttled,
}

pub trait Collector {
    fn collect(&self, registry: &mut Registry) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait Handler {
    fn handle(&self) -> impl Future<Output = anyhow::Result<String>> + Send;
}

impl<Throttled> MetricsHandler<Throttled>
where
    Throttled: Collector,
{
    pub fn new(throttled_collector: Throttled) -> Self {
        Self { throttled: throttled_collector }
    }
}

impl<Throttled> Handler for MetricsHandler<Throttled>
where 
    Throttled: Collector + Send + Sync + 'static,
{
    async fn handle(&self) -> anyhow::Result<String> {
        let mut registry = Registry::default();

        self.throttled.collect(&mut registry).await?;

        let mut buffer = String::new();
        text::encode(&mut buffer, &registry).unwrap();

        Ok(buffer)
    }
}
