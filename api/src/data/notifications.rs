use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use mongodb::{
	Collection,
	bson::doc,
	error::{ErrorKind, WriteFailure::WriteError},
};
use serde::{Deserialize, Serialize};

use crate::utils::{errors::AppError, types::AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
	#[serde(rename = "_id")]
	pub id:             Option<String>,
	#[serde(rename = "content")]
	pub content:        String,
	#[serde(rename = "channel")]
	pub channel:        String, // "push" or "email"
	#[serde(rename = "recipient")]
	pub recipient:      String, // email or device token, but for simplicity, using String
	#[serde(rename = "scheduledTime")]
	pub scheduled_time: u32, // For simplicity, using String (ideally use a DateTime type)
	#[serde(rename = "timezone")]
	pub timezone:       String,
	#[serde(rename = "priority")]
	pub priority:       String,
	#[serde(rename = "status")]
	pub status:         String, // "pending", "sent", "failed", "cancelled"
	#[serde(rename = "retryCount")]
	pub retry_count:    u32,
}

pub struct GetMessagesOptions {
	pub priority: Option<String>,
	pub status:   Option<String>,
	pub limit:    Option<i64>,
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
	async fn create(&self, notification: Notification) -> AppResult<Notification>;
	async fn get_messages(
		&self,
		opts: GetMessagesOptions,
	) -> AppResult<Box<dyn Stream<Item = Result<Notification, AppError>> + Send + Unpin>>;
	async fn update_message_status(&self, id: String, status: String) -> AppResult<()>;
}

pub struct NotificationRepositoryImpl {
	notifications: Collection<Notification>,
}

impl NotificationRepositoryImpl {
	pub fn new(notifications: Collection<Notification>) -> Self {
		NotificationRepositoryImpl { notifications }
	}
}

#[async_trait]
impl NotificationRepository for NotificationRepositoryImpl {
	async fn create(&self, notification: Notification) -> AppResult<Notification> {
		// generate a new UUID for the notification
		let id = Some(mongodb::bson::oid::ObjectId::new().to_hex());
		let notification = Notification { id, ..notification };
		let result = self.notifications.insert_one(notification.clone()).await;
		match result {
			Ok(_) => Ok(notification),
			Err(e) => match e.kind.as_ref() {
				ErrorKind::Write(WriteError(write_error)) if write_error.code == 11000 => {
					Err(AppError::DuplicateKey)
				}
				_ => Err(AppError::RepositoryError(
					"Failed to create notification".to_string(),
				)),
			},
		}
	}

	async fn get_messages(
		&self,
		opts: GetMessagesOptions,
	) -> AppResult<Box<dyn Stream<Item = Result<Notification, AppError>> + Send + Unpin>> {
		let mut filter = doc! {};

		if let Some(priority) = opts.priority {
			filter.insert("priority", priority);
		}

		if let Some(status) = opts.status {
			filter.insert("status", status);
		}

		let limit = opts.limit.unwrap_or(i64::MAX);
		let cursor = self
			.notifications
			.find(filter)
			.limit(limit)
			.await
			.map_err(|_| AppError::RepositoryError("Failed to retrieve messages".to_string()))?;

		let stream = cursor.into_stream();
		let mapped_stream = stream.map(|result| {
			result.map_err(|e| AppError::RepositoryError(format!("Failed to read document: {}", e)))
		});

		Ok(Box::new(mapped_stream))
	}

	async fn update_message_status(&self, id: String, status: String) -> AppResult<()> {
		let filter = doc! { "_id": id };
		let update = doc! { "$set": { "status": status } };
		let result = self
			.notifications
			.update_one(filter, update)
			.await
			.map_err(|e| {
				AppError::RepositoryError(format!("Failed to update with err: {:?}", e))
			})?;

		if result.modified_count == 0 {
			Err(AppError::RepositoryError(
				"No documents matched the query".to_string(),
			))
		} else {
			Ok(())
		}
	}
}
