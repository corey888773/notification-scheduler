use axum::{Router, response::IntoResponse};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use tokio::net::TcpListener;

async fn metrics_handler(metric_handle: PrometheusHandle) -> impl IntoResponse {
	metric_handle.render()
}

pub struct ServerOptions {
	pub host:          String,
	pub port:          String,
	pub metric_handle: PrometheusHandle,
}

pub async fn create_metrics_router(opts: ServerOptions) -> (Router, TcpListener) {
	let listener = TcpListener::bind(format!("{}:{}", opts.host, opts.port))
		.await
		.expect("Failed to bind to address");
	let router = setup_routes(opts.metric_handle);
	(router, listener)
}

fn setup_routes(metric_handle: PrometheusHandle) -> Router {
	Router::new().route(
		"/metrics",
		axum::routing::get({
			let metric_handle = metric_handle.clone();
			move || async move { metrics_handler(metric_handle.clone()).await }
		}),
	)
}
