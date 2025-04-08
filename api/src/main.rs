#![deny(unused_imports)]

use std::{env, time::Duration};

use common::{axum, axum_prometheus::PrometheusMetricLayer, monitoring, tokio};
use env_logger::{Builder, Env, Target};
use log::info;

use crate::{crone::build_notification_scheduler_future, data::notifications::Priority};

mod api;
mod app_state;
mod crone;
mod data;
mod messaging;
mod server;
mod services;
mod utils;

#[tokio::main]
async fn main() {
	// Load environment variables from the .env file
	dotenvy::from_filename("api/src/app.env").ok();

	// Initialize the logger
	println!("RUST_LOG: {}", env::var("RUST_LOG").unwrap());
	Builder::from_env(Env::default())
		.target(Target::Stdout)
		.init();

	// Get environment variables
	let port: String = env::var("APP_PORT").unwrap_or("8080".to_string());
	let prometheus_port: String = env::var("PROMETHEUS_PORT").unwrap_or("9090".to_string());
	let mongo_uri: String =
		env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());
	let nats_url: String = env::var("NATS_URL").unwrap_or("localhost:4222".to_string());
	let host: String = env::var("HOST").unwrap_or("0.0.0.0".to_string());

	// Create the application state
	let app_state = app_state::AppState::new(app_state::AppStateOptions {
		mongo_url: mongo_uri.clone(),
		nats_url:  nats_url.clone(),
	})
	.await;

	// Set up Prometheus metrics
	let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
	let (mut app, app_listener) = server::server::create_server(
		server::server::ServerOptions {
			host: host.clone(),
			port: port.clone(),
		},
		app_state.clone(),
	)
	.await;
	app = app.layer(prometheus_layer.clone());
	let (prometheus, prometheus_listener) =
		monitoring::server::create_metrics_router(monitoring::server::ServerOptions {
			host:          host.clone(),
			port:          prometheus_port.clone(),
			metric_handle: metric_handle.clone(),
		})
		.await;

	// Start the servers
	let low_priority_scheduler_future = build_notification_scheduler_future(
		app_state.clone(),
		Priority::Low,
		Duration::from_secs(5),
	);

	let high_priority_scheduler_future = build_notification_scheduler_future(
		app_state.clone(),
		Priority::High,
		Duration::from_secs(1),
	);

	let app_future = async {
		info!("Server running on port: {}", port);
		axum::serve(app_listener, app)
			.await
			.expect("Server failed to start");
	};

	let prom_future = async {
		info!("Prometheus server running on port: {}", prometheus_port);
		axum::serve(prometheus_listener, prometheus)
			.await
			.expect("Prometheus server failed to start");
	};

	let _ = tokio::join!(
		app_future,
		prom_future,
		low_priority_scheduler_future,
		high_priority_scheduler_future
	);
}
