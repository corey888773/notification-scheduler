use common::axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use serde::Serialize;

pub struct AppResponse<T: Serialize> {
	pub status: StatusCode,
	pub data:   T, // Store unwrapped data
}

impl<T: Serialize> AppResponse<T> {
	pub fn new_with_data(status: StatusCode, data: T) -> Self {
		AppResponse { status, data }
	}
}

// Special implementation for empty responses
impl AppResponse<serde_json::Value> {
	pub fn new(status: StatusCode) -> Self {
		AppResponse {
			status,
			data: serde_json::json!({}),
		}
	}
}

impl<T: Serialize> IntoResponse for AppResponse<T> {
	fn into_response(self) -> Response {
		// Single serialization point
		(self.status, Json(self.data)).into_response()
	}
}
