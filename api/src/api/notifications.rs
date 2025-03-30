use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};

use crate::{api::common::AppResponse, app_state::AppState, data::notifications::Notification};

pub fn routes(state: Arc<AppState>) -> Router {
	let routes = Router::new().route("/", post(create)).with_state(state);

	Router::new().nest("/notifications", routes)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateRequest {
	#[serde(flatten)]
	notification: Notification,
}

#[axum::debug_handler]
async fn create(state: State<Arc<AppState>>, req: Json<CreateRequest>) -> impl IntoResponse {
	let service = state.notification_service.as_ref();
	let notification = req.notification.clone();
	match service.create_notification(notification).await {
		Ok(_) => AppResponse::new(StatusCode::CREATED).into_response(),
		Err(e) => e.into_response(),
	}
}
