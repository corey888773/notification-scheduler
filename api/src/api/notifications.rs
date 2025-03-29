use crate::{app_state::AppState, data::notifications::Notification};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::http::StatusCode;
use crate::api::common::AppResponse;

pub fn routes(state: Arc<AppState>) -> Router {
	let routes = Router::new().route("/", post(create)).with_state(state);

	Router::new().nest("/notifications", routes)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateRequest {
	#[serde(flatten)]
	notification: Notification,
}
async fn create(state: State<Arc<AppState>>, req: Json<CreateRequest>) -> impl IntoResponse {
	println!("->> POST {:<20}", "/notifications");
	let service = state.notification_service.clone();
	let notification = req.notification.clone();
	match service.create_notification(notification).await {
		Ok(_) => AppResponse::new(StatusCode::CREATED).into_response(),
		Err(e) => e.into_response(),
	}
}
