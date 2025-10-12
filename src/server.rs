use std::{net::Ipv4Addr, sync::Arc};

use axum::{extract::State, http::{header::CONTENT_TYPE, StatusCode}, response::IntoResponse, routing::get, Router};
use tokio::{net::TcpListener, signal::unix::{self, SignalKind}};

use crate::metrics::{Handler};

pub struct Server<MetricsHandler> {
    port: u16,
    metrics_handler: MetricsHandler,
}

impl<MetricsHandler> Server<MetricsHandler>
where
    MetricsHandler: Handler + Send + Sync + 'static,
{
    pub fn new(port: u16, metrics_handler: MetricsHandler) -> Self {
        Self {
            port,
            metrics_handler,
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/metrics", get(handle))
            .with_state(Arc::new(self.metrics_handler));

        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, self.port)).await?;

        tracing::info!("listening on {}", listener.local_addr()?);

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

#[tracing::instrument(skip_all)]
async fn handle<S>(State(service): State<Arc<S>>) -> impl IntoResponse
where
    S: Handler,
{
    match service.handle().await {
        Ok(res) => (
            StatusCode::OK,
            [(CONTENT_TYPE, "application/openmetrics-text; version=1.0.0; charset=utf-8")],
            res,
        ).into_response(),
        Err(err) => {
            tracing::error!("{err:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
        },
    }
}

async fn shutdown_signal() {
    let mut sigint = unix::signal(SignalKind::interrupt()).expect("SIGINT error");
    let mut sigterm = unix::signal(SignalKind::terminate()).expect("SIGTERM error");

    tokio::select! {
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {},
    }
}
