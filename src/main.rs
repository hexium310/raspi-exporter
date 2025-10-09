use crate::{
    metrics::{
        throttled::{Throttled, ThrottledExecutor, ThrottledParser},
        MetricsHandler,
    },
    server::Server,
};

mod command;
mod metrics;
mod server;

#[tokio::main]
async fn main() {
    let throttled = Throttled::new(ThrottledExecutor::new("vcgencmd", ["get_throttled"]), ThrottledParser);
    let metrics_handler = MetricsHandler::new(throttled);

    let server = Server::new(8021, metrics_handler);
    server.start().await.unwrap();
}
