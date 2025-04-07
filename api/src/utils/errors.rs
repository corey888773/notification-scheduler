use common::axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
	#[error("Duplicate key")]
	DuplicateKey,
	#[error("Validation error: {0}")]
	ValidationError(String),
	#[error("Service error: {0}")]
	ServiceError(String),
	#[error("Data access error: {0}")]
	RepositoryError(String),
	#[error("Serialization error: {0}")]
	SerialError(String),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, error_message) = match &self {
			AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
			AppError::ServiceError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
			AppError::RepositoryError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
			AppError::DuplicateKey => (StatusCode::CONFLICT, self.to_string()),
			AppError::SerialError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
		};
		let body = Json(serde_json::json!({ "error": error_message }));
		(status, body).into_response()
	}
}
