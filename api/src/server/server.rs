use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use crate::{
	app,
	app_state::AppState,
	data,
	data::notifications::NotificationRepository,
	services,
	services::notifications::NotificationService,
};

pub struct ServerOptions {
	pub host:      String,
	pub port:      String,
	pub mongo_url: String,
}

pub async fn create_server(opts: ServerOptions) -> (Router, TcpListener) {
	let db = data::db::DbContext::new(opts.mongo_url.as_ref())
		.await
		.expect("Failed to set up database");

	let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
		data::notifications::NotificationRepositoryImpl::new(db.notifications_collection),
	);
	let notification_service: Arc<dyn NotificationService> = Arc::new(
		services::notifications::NotificationServiceImpl::new(notification_repository),
	);
	let app_state = AppState::new(notification_service);
	let app = app(app_state);
	let listener = TcpListener::bind(format!("{}:{}", opts.host, opts.port))
		.await
		.expect("Failed to bind to address");

	(app, listener)
}
