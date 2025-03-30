use axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};

pub struct AppResponse {
	pub status: StatusCode,
	pub body:   String,
}

impl AppResponse {
	pub fn new_body(status: StatusCode, body: String) -> Self {
		AppResponse { status, body }
	}

	pub fn new(status: StatusCode) -> Self {
		AppResponse {
			status,
			body: "".to_string(),
		}
	}
}

impl IntoResponse for AppResponse {
	fn into_response(self) -> Response {
		(self.status, Json(self.body)).into_response()
	}
}
