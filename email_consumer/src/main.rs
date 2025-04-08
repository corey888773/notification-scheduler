use std::{env, sync::Arc};

use common::{axum_prometheus::metrics::set_global_recorder, monitoring};
use env_logger::{Builder, Env, Target};
use log::info;

use crate::consumer::service::NatsConsumerOptions;

mod app_state;
mod consumer;
mod handler;
mod metrics;

const CHANNEL: &str = "email";

#[tokio::main]
async fn main() {
	dotenvy::from_filename("email_consumer/app.env").ok();
	println!("RUST_LOG: {}", env::var("RUST_LOG").unwrap());
	Builder::from_env(Env::default())
		.target(Target::Stdout)
		.init();

	let nats_url = env::var("NATS_URL").unwrap_or("localhost:4222".to_string());
	let prometheus_port: String = env::var("PROMETHEUS_PORT").unwrap_or("9090".to_string());
	let host: String = env::var("HOST").unwrap_or("0.0.0.0".to_string());
	let recipient_id = env::var("RECIPIENT_ID").unwrap_or("email_consumer".to_string());

	let (prometheus_layer, prometheus_recorder) = monitoring::server::default_pair();
	let metrics = Arc::new(metrics::metrics::setup_metrics(prometheus_recorder.clone()));
	set_global_recorder(prometheus_recorder.clone()).expect("Failed to set global recorder");

	let app_state = Arc::new(app_state::AppState {
		metrics: metrics.clone(),
	});

	let filter_subject = format!("notifications_{}.{}", CHANNEL, recipient_id);
	let mut nats_consumer = consumer::service::NatsConsumer::new(NatsConsumerOptions {
		nats_url:       nats_url.clone(),
		recipient_id:   recipient_id.clone(),
		filter_subject: filter_subject.clone(),
		channel:        CHANNEL.to_string(),
	})
	.await;

	let (mut prometheus, prometheus_listener) =
		monitoring::server::create_metrics_router(monitoring::server::ServerOptions {
			host:          host.clone(),
			port:          prometheus_port.clone(),
			metric_handle: prometheus_recorder.clone().handle(),
		})
		.await;
	prometheus = prometheus.layer(prometheus_layer);

	let prom_future = async {
		info!("Prometheus server running on port: {}", prometheus_port);
		axum::serve(prometheus_listener, prometheus)
			.await
			.expect("Prometheus server failed to start");
	};

	let nats_future = async move {
		let message_handler: Box<dyn handler::MessageHandler> =
			Box::new(handler::std_out::StdOutHandlerImpl {
				app_state: app_state.clone(),
			});

		info!("Nats consumer started for subject: {}", filter_subject);
		nats_consumer.start(message_handler).await
	};

	let (_res1, _res2) = tokio::join!(prom_future, nats_future);
}
