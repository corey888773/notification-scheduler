use std::sync::Arc;

use axum::routing::get;
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
		.route("/", get(get_all))
		.route("/{id}", delete(cancel))
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
	// there should be validation for the notification, but I am too lazy to do it
	// now :)
	match service
		.clone()
		.create_notification(notification.clone(), req.force)
		.await
	{
		Ok(id) => AppResponse::new_with_data(StatusCode::CREATED, id).into_response(),
		Err(e) => e.into_response(),
	}
}

#[debug_handler]
async fn cancel(state: State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
	let service = state.notification_service.clone();
	match service.cancel_notification(id).await {
		Ok(_) => AppResponse::new(StatusCode::OK).into_response(),
		Err(e) => e.into_response(),
	}
}

#[debug_handler]
async fn get_all(state: State<Arc<AppState>>) -> impl IntoResponse {
	let service = state.notification_service.clone();
	match service.get_all().await {
		Ok(notifications) => {
			AppResponse::new_with_data(StatusCode::OK, notifications).into_response()
		}
		Err(e) => e.into_response(),
	}
}
