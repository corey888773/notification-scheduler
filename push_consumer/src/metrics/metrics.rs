use std::sync::Arc;

use common::{
	axum_prometheus::metrics_exporter_prometheus::PrometheusRecorder,
	monitoring::metrics::Metrics,
};

pub const PUSH_CONSUMER_CONSUMED_MESSAGES: &str = "push_consumer_consumed_messages";

pub fn setup_metrics(prometheus_recorder: Arc<PrometheusRecorder>) -> Metrics {
	let metrics = Metrics::new(prometheus_recorder);
	metrics
		.register_counter(
			PUSH_CONSUMER_CONSUMED_MESSAGES,
		)
		.unwrap();

	metrics
}
