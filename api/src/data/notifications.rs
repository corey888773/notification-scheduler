use async_trait::async_trait;
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
	id:             Option<String>,
	#[serde(rename = "content")]
	content:        String,
	#[serde(rename = "channel")]
	channel:        String, // "push" or "email"
	#[serde(rename = "recipient")]
	recipient:      String, // email or device token, but for simplicity, using String
	#[serde(rename = "scheduledTime")]
	scheduled_time: u32, // For simplicity, using String (ideally use a DateTime type)
	#[serde(rename = "timezone")]
	timezone:       String,
	#[serde(rename = "priority")]
	priority:       String,
	#[serde(rename = "status")]
	status:         String, // "pending", "sent", "failed", "cancelled"
	#[serde(rename = "retryCount")]
	retry_count:    u32,
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
	async fn create(&self, notification: Notification) -> AppResult<Notification>;
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
}
