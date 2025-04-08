use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::{Stream, StreamExt, TryStreamExt};
use mongodb::{
	Collection,
	bson,
	bson::{doc, oid::ObjectId},
	error::{ErrorKind, WriteFailure::WriteError},
};
use serde::{Deserialize, Serialize};

use crate::utils::{errors::AppError, types::AppResult};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Channel {
	#[serde(rename = "push")]
	Push,
	#[serde(rename = "email")]
	Email,
}
impl From<String> for Channel {
	fn from(s: String) -> Self {
		match s.as_str() {
			"push" => Channel::Push,
			"email" => Channel::Email,
			_ => panic!("Invalid notification type"),
		}
	}
}
impl From<Channel> for String {
	fn from(val: Channel) -> Self {
		match val {
			Channel::Push => "push".to_string(),
			Channel::Email => "email".to_string(),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Status {
	#[serde(rename = "pending")]
	Pending,
	#[serde(rename = "sent")]
	Sent,
	#[serde(rename = "failed")]
	Failed,
	#[serde(rename = "cancelled")]
	Cancelled,
}

impl From<String> for Status {
	fn from(s: String) -> Self {
		match s.as_str() {
			"pending" => Status::Pending,
			"sent" => Status::Sent,
			"failed" => Status::Failed,
			"cancelled" => Status::Cancelled,
			_ => panic!("Invalid notification status"),
		}
	}
}

impl From<Status> for String {
	fn from(val: Status) -> Self {
		match val {
			Status::Pending => "pending".to_string(),
			Status::Sent => "sent".to_string(),
			Status::Failed => "failed".to_string(),
			Status::Cancelled => "cancelled".to_string(),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Priority {
	#[serde(rename = "high")]
	High,
	#[serde(rename = "low")]
	Low,
}

impl From<String> for Priority {
	fn from(s: String) -> Self {
		match s.as_str() {
			"high" => Priority::High,
			"low" => Priority::Low,
			_ => panic!("Invalid notification priority"),
		}
	}
}

impl From<Priority> for String {
	fn from(val: Priority) -> Self {
		match val {
			Priority::High => "high".to_string(),
			Priority::Low => "low".to_string(),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recipient {
	#[serde(rename = "id")]
	pub id:              String,
	#[serde(rename = "timezone_offset")]
	pub timezone_offset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
	#[serde(rename = "_id")]
	pub id:             Option<String>,
	#[serde(rename = "content")]
	pub content:        String,
	#[serde(rename = "channel")]
	pub channel:        Channel, // "push" or "email"
	#[serde(rename = "recipient")]
	pub recipient:      Recipient, // email or device token, but for simplicity, using String
	#[serde(rename = "scheduledTime")]
	pub scheduled_time: DateTime<Utc>,
	#[serde(rename = "priority")]
	pub priority:       Priority,
	#[serde(rename = "status")]
	pub status:         Status, // "pending", "sent", "failed", "cancelled"
	#[serde(rename = "retryCount")]
	pub retry_count:    u32,
}

#[derive(Default)]
pub struct GetMessagesOptions {
	pub priority:          Option<Priority>,
	pub status:            Option<Status>,
	pub limit:             Option<i64>,
	pub scheduled_time:    Option<DateTime<Utc>>,
	pub respect_nighttime: Option<bool>,
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
		let id = Some(ObjectId::new().to_hex());
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
		let mut match_stage = doc! {};
		let mut pipeline = vec![];

		if let Some(priority) = opts.priority {
			let priority_str: String = priority.into();
			match_stage.insert("priority", priority_str);
		}

		if let Some(status) = opts.status {
			let status_str: String = status.into();
			match_stage.insert("status", status_str);
		}

		if let Some(scheduled_time) = opts.scheduled_time {
			let scheduled_time_str = scheduled_time.to_rfc3339();
			match_stage.insert("scheduledTime", doc! { "$lte": scheduled_time_str.clone() });

			if opts.respect_nighttime.unwrap_or(false) {
				let start_hour = 8;
				let end_hour = 22;

				pipeline.push(doc! {
					"$addFields": {
						"userLocalHour": {
							"$add": [
								{"$hour": {"$toDate": scheduled_time_str }},
								{"$divide": [
									{"$toInt": {"$substr": ["$recipient.timezone_offset", 0, 3]}},
									1
								]}
							]
						}
					}
				});
				pipeline.push(doc! {
					"$match": {
						"userLocalHour": {
							"$gte": start_hour,
							"$lt": end_hour
						}
					}
				});
			}
		}

		pipeline.push(doc! { "$match": match_stage });
		pipeline.push(doc! { "$limit": opts.limit.unwrap_or(i64::MAX) });
		let cursor = self
			.notifications
			.aggregate(pipeline)
			.await
			.map_err(|e| AppError::RepositoryError(format!("Failed to aggregate: {}", e)))?;

		let stream = cursor.into_stream();
		let mapped_stream = stream.map(|result| {
			result
				.map_err(|e| AppError::RepositoryError(format!("Failed to read document: {}", e)))
				.and_then(|doc| {
					bson::from_document::<Notification>(doc)
						.map_err(|e| AppError::SerialError(format!("Failed to deserialize: {}", e)))
				})
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
