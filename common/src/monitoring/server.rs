use std::sync::Arc;
use axum::{Router, response::IntoResponse};
use axum_prometheus::metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle, PrometheusRecorder};
use axum_prometheus::{PrometheusMetricLayer, AXUM_HTTP_REQUESTS_DURATION_SECONDS};
use axum_prometheus::utils::SECONDS_DURATION_BUCKETS;
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

pub fn default_pair<'a>() -> (PrometheusMetricLayer<'a>, Arc<PrometheusRecorder>) {
	let prometheus_recorder = Arc::new(PrometheusBuilder::new()
		.set_buckets_for_metric(
			Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
			SECONDS_DURATION_BUCKETS,
		).unwrap().build_recorder());

	let prometheus_layer = PrometheusMetricLayer::new();
	(prometheus_layer, prometheus_recorder)
}