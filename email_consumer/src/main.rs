use std::{env, sync::Arc};

use common::{axum_prometheus::metrics::set_global_recorder, monitoring};

mod app_state;
mod consumer;
mod metrics;

#[tokio::main]
async fn main() {
	dotenvy::from_filename("email_consumer/app.env").ok();
	let nats_url = env::var("NATS_URL").unwrap_or("localhost:4222".to_string());
	let prometheus_port: String = env::var("PROMETHEUS_PORT").unwrap_or("9090".to_string());
	let host: String = env::var("HOST").unwrap_or("0.0.0.0".to_string());

	let (prometheus_layer, prometheus_recorder) = monitoring::server::default_pair();
	let metrics = Arc::new(metrics::metrics::setup_metrics(prometheus_recorder.clone()));
	set_global_recorder(prometheus_recorder.clone()).expect("Failed to set global recorder");

	let app_state = Arc::new(app_state::AppState {
		metrics: metrics.clone(),
	});

	let mut nats_consumer = consumer::service::NatsConsumer::new(nats_url, app_state).await;
	let (mut prometheus, prometheus_listener) =
		monitoring::server::create_metrics_router(monitoring::server::ServerOptions {
			host:          host.clone(),
			port:          prometheus_port.clone(),
			metric_handle: prometheus_recorder.clone().handle(),
		})
		.await;
	prometheus = prometheus.layer(prometheus_layer);

	let prom_future = async {
		println!("Prometheus server running on port: {}", prometheus_port);
		axum::serve(prometheus_listener, prometheus)
			.await
			.expect("Prometheus server failed to start");
	};
	let kafka_future = async {
		println!("Nats consumer started");
		nats_consumer.start().await
	};

	let (_res1, _res2) = tokio::join!(prom_future, kafka_future);
}
