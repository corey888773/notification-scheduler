use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::tokio::time::sleep;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error, info};

use crate::{
	data::{
		notifications,
		notifications::{GetMessagesOptions, Notification, Priority, Status},
	},
	messaging::broker,
	utils::{errors::AppError, types::AppResult},
};

const TESTING_ERROR_RATIO: u8 = 50; // 50% chance of error

#[derive(Clone)]
pub struct NotificationServiceImpl {
	repository: Arc<dyn notifications::NotificationRepository>,
	broker:     Arc<dyn broker::Broker>,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
	async fn create_notification(
		&self,
		notification: Notification,
		force: Option<bool>,
	) -> AppResult<String>;
	async fn cancel_notification(&self, id: String) -> AppResult<()>;
	async fn send_messages(
		&self,
		priority: Priority,
		scheduled_time: DateTime<Utc>,
	) -> AppResult<()>;
	async fn get_all(&self) -> AppResult<Vec<Notification>>;
}

impl NotificationServiceImpl {
	pub fn new(
		repository: Arc<dyn notifications::NotificationRepository>,
		broker: Arc<dyn broker::Broker>,
	) -> Self {
		NotificationServiceImpl { repository, broker }
	}
}

impl NotificationServiceImpl {
	// Simulate a random error for demonstration purposes
	async fn wrapped_send_individual_message(&self, message: Notification) -> AppResult<()> {
		let random_number = rand::random::<u8>() % 100;
		if random_number < TESTING_ERROR_RATIO {
			Err(AppError::ServiceError("Random error occurred".to_string()))
		} else {
			self.send_individual_message(message).await
		}
	}

	async fn update_message_status(&self, id: String, status: Status) -> AppResult<()> {
		self.repository.update_message_status(id, status).await
	}

	async fn send_individual_message(&self, message: Notification) -> AppResult<()> {
		let message_string =
			serde_json::to_string(&message).map_err(|e| AppError::ServiceError(e.to_string()))?;
		self.broker
			.send_message(
				message.channel.into(),
				message.recipient.id,
				message_string,
				message.id.unwrap(),
			)
			.await
	}

	async fn try_send_message(&self, message: Notification) -> AppResult<bool> {
		let mut attempts = 0;
		let mut sent = false;

		while attempts < 3 {
			attempts += 1;

			debug!(
				"Sending message attempt {} for message {:?}",
				attempts, message
			);
			match self.wrapped_send_individual_message(message.clone()).await {
				Ok(_) => {
					info!("Message sent successfully: {:?}", message);
					sent = true;
					break;
				}
				Err(e) => {
					error!("Error sending message: {:?}", e);
					sleep(Duration::from_secs(1)).await;
				}
			}
		}

		if let Some(id) = message.id.clone() {
			let status = if sent { Status::Sent } else { Status::Failed };
			self.repository.update_message_status(id, status).await?;
		}

		Ok(sent)
	}
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
	async fn create_notification(
		&self,
		notification: Notification,
		force: Option<bool>,
	) -> AppResult<String> {
		let notification = self.repository.create(notification.clone()).await?;

		if force.unwrap_or(false) {
			let _ = self.try_send_message(notification.clone()).await?;
		}

		Ok(notification.id.unwrap())
	}

	async fn cancel_notification(&self, id: String) -> AppResult<()> {
		self.repository
			.update_message_status(id, Status::Cancelled)
			.await
	}

	async fn send_messages(
		&self,
		priority: Priority,
		scheduled_time: DateTime<Utc>,
	) -> AppResult<()> {
		let pending_messages = self
			.repository
			.get_messages(GetMessagesOptions {
				priority:          Some(priority),
				status:            Some(Status::Pending),
				limit:             Some(10),
				scheduled_time:    Some(scheduled_time),
				respect_nighttime: Some(true),
			})
			.await?;

		let tasks = FuturesUnordered::from_iter(
			pending_messages
				.map(|result_message| {
					let this = self.clone();
					async move {
						match result_message {
							Ok(message) => this.try_send_message(message).await,
							Err(e) => Err(e),
						}
					}
				})
				.collect::<Vec<_>>()
				.await,
		);

		tasks
			.for_each(|res| async {
				if let Err(e) = res {
					error!("Error processing message: {:?}", e);
				}
			})
			.await;

		Ok(())
	}

	async fn get_all(&self) -> AppResult<Vec<Notification>> {
		let stream = self
			.repository
			.get_messages(GetMessagesOptions::default())
			.await?;

		let mut notifications = Vec::new();
		let results: Vec<Result<Notification, AppError>> = stream.collect().await;
		for result in results {
			match result {
				Ok(notification) => notifications.push(notification),
				Err(e) => return Err(e), // Return first error encountered
			}
		}

		Ok(notifications)
	}
}
