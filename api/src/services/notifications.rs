use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::time::sleep;

use crate::{
	data::notifications,
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
	async fn create_notification(&self, notification: notifications::Notification)
	-> AppResult<()>;
	async fn send_messages(&self, priority: String) -> AppResult<()>;
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
	async fn send_individual_message(&self, message: notifications::Notification) -> AppResult<()> {
		let message_string =
			serde_json::to_string(&message).map_err(|e| AppError::ServiceError(e.to_string()))?;
		self.broker
			.send_message(&message.channel, &message_string, "")
			.await
	}

	async fn update_message_status(&self, id: String, status: String) -> AppResult<()> {
		self.repository.update_message_status(id, status).await
	}
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
	async fn create_notification(
		&self,
		notification: notifications::Notification,
	) -> AppResult<()> {
		let creation_result = self.repository.create(notification.clone()).await;

		let created_notif = match creation_result {
			Ok(notification) => notification,
			Err(e) => {
				return Err(e);
			}
		};

		let notification_string = serde_json::to_string(&created_notif).unwrap();
		self.broker
			.send_message("email", &notification_string, "")
			.await
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
									println!(
										"Sending message: {:?}, attempt {:?}",
										&message, attempts
									);
									match this.send_individual_message(message.clone()).await {
										Ok(_) => {
											sent = true;
											break;
										}
										Err(e) => {
											eprintln!(
												"Attempt {} failed for message {}: {:?}",
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
}
