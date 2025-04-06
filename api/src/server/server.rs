use std::sync::Arc;

use common::{axum::Router, tokio::net::TcpListener};

use crate::{api, app_state::AppState};

pub struct ServerOptions {
	pub host: String,
	pub port: String,
}

pub async fn create_server(opts: ServerOptions, app_state: Arc<AppState>) -> (Router, TcpListener) {
	let app = setup_routes(app_state);
	let listener = TcpListener::bind(format!("{}:{}", opts.host, opts.port))
		.await
		.expect("Failed to bind to address");

	(app, listener)
}

fn setup_routes(app_state: Arc<AppState>) -> Router {
	let api_routes = Router::new().merge(api::notifications::routes(app_state.clone()));
	Router::new().nest("/api/v1", api_routes)
}
