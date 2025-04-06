use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use common::tokio::time::sleep;
use futures::stream::{FuturesUnordered, StreamExt};

use crate::{
	data::{notifications, notifications::Notification},
	messaging::broker,
	utils::{errors::AppError, types::AppResult},
};

#[derive(Clone)]
pub struct NotificationServiceImpl {
	repository: Arc<dyn notifications::NotificationRepository>,
	broker:     Arc<dyn broker::Broker>,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
	async fn create_notification(&self, notification: Notification) -> AppResult<String>;
	async fn stop_notification(&self, id: String) -> AppResult<()>;
	async fn send_messages(&self, priority: String) -> AppResult<()>;
	async fn send_individual_message(&self, message: Notification) -> AppResult<()>;
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
		let random_number = rand::random::<u8>() % 3;
		if random_number == 0 {
			Err(AppError::ServiceError("Random error occurred".to_string()))
		} else {
			println!("Sending message: {:?}", message.clone().id.unwrap());
			self.send_individual_message(message).await
		}
	}

	async fn update_message_status(&self, id: String, status: String) -> AppResult<()> {
		self.repository.update_message_status(id, status).await
	}
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
	async fn create_notification(&self, notification: Notification) -> AppResult<String> {
		let creation_result = self.repository.create(notification.clone()).await;
		match creation_result {
			Ok(notification) => Ok(notification.id.unwrap()),
			Err(e) => Err(e),
		}
	}

	async fn stop_notification(&self, id: String) -> AppResult<()> {
		self.update_message_status(id, "stopped".to_string()).await
	}

	async fn send_messages(&self, priority: String) -> AppResult<()> {
		let pending_messages = self
			.repository
			.get_messages(notifications::GetMessagesOptions {
				priority: Some(priority),
				status:   Some("pending".to_string()),
				limit:    Some(10),
			})
			.await?;

		let tasks = FuturesUnordered::from_iter(
			pending_messages
				.map(|result_message| {
					let this = self.clone();
					async move {
						match result_message {
							Ok(message) => {
								let mut attempts = 0;
								let mut sent = false;

								while attempts < 3 {
									attempts += 1;
									println!("attempt {:?}", attempts);
									match this
										.wrapped_send_individual_message(message.clone())
										.await
									{
										Ok(_) => {
											sent = true;
											break;
										}
										Err(e) => {
											eprintln!(
												"Attempt {} failed for message {:?}: {:?}",
												attempts,
												message.id.clone().unwrap(),
												e
											);
											sleep(Duration::from_secs(1)).await;
										}
									}
								}

								let status = if sent {
									"sent".to_string()
								} else {
									"failed".to_string()
								};
								if let Some(id) = message.id.clone() {
									this.update_message_status(id, status).await?;
								}
								Ok(())
							}
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
					eprintln!("Error processing message: {:?}", e);
				}
			})
			.await;

		Ok(())
	}

	async fn send_individual_message(&self, message: Notification) -> AppResult<()> {
		let message_string =
			serde_json::to_string(&message).map_err(|e| AppError::ServiceError(e.to_string()))?;
		self.broker
			.send_message(message.channel, message_string, message.id.unwrap())
			.await
	}

	async fn get_all(&self) -> AppResult<Vec<Notification>> {
		let stream = self
			.repository
			.get_messages(notifications::GetMessagesOptions {
				priority: None,
				status:   None,
				limit:    None,
			})
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
