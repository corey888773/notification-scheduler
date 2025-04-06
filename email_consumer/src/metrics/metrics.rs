use std::sync::Arc;
use common::axum_prometheus::metrics_exporter_prometheus::PrometheusRecorder;
use common::monitoring::metrics::Metrics;

pub const EMAIL_CONSUMER_CONSUMED_MESSAGES: &str = "email_consumer_consumed_messages";

pub fn setup_metrics(prometheus_recorder: Arc<PrometheusRecorder>) -> Metrics {
    let metrics = Metrics::new(prometheus_recorder);
    metrics
        .register_counter(EMAIL_CONSUMER_CONSUMED_MESSAGES, "Count of consumed messages")
        .unwrap();

    metrics
}