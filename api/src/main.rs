#![deny(unused_imports)]

use std::{env, time::Duration};

use common::{axum, axum_prometheus::PrometheusMetricLayer, monitoring, tokio};

use crate::crone::scheduler::CronScheduler;

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
	dotenvy::from_filename("api/src/app.env").ok();
	let port: String = env::var("APP_PORT").unwrap_or("8080".to_string());
	let prometheus_port: String = env::var("PROMETHEUS_PORT").unwrap_or("9090".to_string());
	let mongo_uri: String =
		env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());
	let nats_url: String = env::var("NATS_URL").unwrap_or("localhost:4222".to_string());
	let host: String = env::var("HOST").unwrap_or("0.0.0.0".to_string());

	let app_state = app_state::AppState::new(app_state::AppStateOptions {
		mongo_url: mongo_uri.clone(),
		nats_url:  nats_url.clone(),
	})
	.await;

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
	let scheduler = crone::scheduler::CronSchedulerImpl::new(Duration::from_secs(5), move || {
		let notification_service = app_state.notification_service.clone();
		async move {
			println!("Scheduler running");
			notification_service
				.send_messages("low".to_string())
				.await
				.unwrap();
		}
	});

	let app_future = async {
		println!("Server running on port: {}", port);
		axum::serve(app_listener, app)
			.await
			.expect("Server failed to start");
	};

	let prom_future = async {
		println!("Prometheus server running on port: {}", prometheus_port);
		axum::serve(prometheus_listener, prometheus)
			.await
			.expect("Prometheus server failed to start");
	};

	let scheduler_future = async {
		scheduler.start().await;
	};

	let (_res1, _res2, _res3) = tokio::join!(app_future, prom_future, scheduler_future);
}
