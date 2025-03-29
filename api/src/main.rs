use crate::{
	app_state::AppState, data::notifications::NotificationRepository,
	services::notifications::NotificationService,
};
use axum::{Router, routing::get};
use std::{env, sync::Arc};

mod api;
mod app_state;
mod data;
mod services;
mod utils;

#[tokio::main]
async fn main() {
	dotenvy::from_filename("api/src/app.env").ok();
	let port: String = env::var("PORT").unwrap_or("8080".to_string());
	let db = data::db::DbContext::new(&env::var("MONGO_URL").unwrap())
		.await
		.unwrap();
	println!("Connected to MongoDB");

	let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
		data::notifications::NotificationRepositoryImpl::new(db.notifications_collection),
	);
	let notification_service: Arc<dyn NotificationService> = Arc::new(
		services::notifications::NotificationServiceImpl::new(notification_repository),
	);
	let app_state = AppState::new(notification_service);
	let app = app(app_state);
	let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
		.await
		.expect("Failed to bind port");

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
