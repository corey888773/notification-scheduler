use std::sync::Arc;

use common::axum::{
	Json,
	Router,
	debug_handler,
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	routing::{delete, post},
};
use serde::{Deserialize, Serialize};

use crate::{api::common::AppResponse, app_state::AppState, data::notifications::Notification};

pub fn routes(state: Arc<AppState>) -> Router {
	let routes = Router::new()
		.route("/", post(create))
		.route("/{id}", delete(stop))
		.with_state(state);

	Router::new().nest("/notifications", routes)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateRequest {
	#[serde(flatten)]
	notification: Notification,
	#[serde(rename = "force")]
	force:        Option<bool>,
}

#[debug_handler]
async fn create(state: State<Arc<AppState>>, req: Json<CreateRequest>) -> impl IntoResponse {
	let service = state.notification_service.clone();
	let notification = req.notification.clone();
	match service
		.clone()
		.create_notification(notification.clone())
		.await
	{
		Ok(_) => AppResponse::new(StatusCode::CREATED).into_response(),
		Err(e) => e.into_response(),
	}
}

#[debug_handler]
async fn stop(state: State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
	let service = state.notification_service.clone();
	match service.stop_notification(id).await {
		Ok(_) => AppResponse::new(StatusCode::OK).into_response(),
		Err(e) => e.into_response(),
	}
}
