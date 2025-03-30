use std::{env, sync::Arc};

use axum::{Router, routing::get};

use crate::{
	app_state::AppState,
	data::notifications::NotificationRepository,
	services::notifications::NotificationService,
};

mod api;
mod app_state;
mod comms;
mod data;
mod server;
mod services;
mod utils;

#[tokio::main]
async fn main() {
	dotenvy::from_filename("api/src/app.env").ok();
	let port: String = env::var("PORT").unwrap_or("8080".to_string());
	let mongo_uri: String =
		env::var("MONGO_URI").unwrap_or("mongodb://localhost:27017".to_string());
	let host: String = "0.0.0.0".to_string();

	let (app, listener) = server::server::create_server(server::server::ServerOptions {
		host:      host.clone(),
		port:      port.clone(),
		mongo_url: mongo_uri.clone(),
	})
	.await;

	println!("Server running on port: {}", port);
	axum::serve(listener, app)
		.await
		.expect("Server failed to start");
}

pub fn app(app_state: Arc<AppState>) -> Router {
	let api_routes = Router::new()
		.route("/hello", get(|| async { "Hello, World!" }))
		.merge(api::notifications::routes(app_state.clone()));
	Router::new().nest("/api/v1", api_routes)
}
