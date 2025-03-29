use crate::{data::notifications, utils::types::AppResult};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct NotificationServiceImpl {
	repository: Arc<dyn notifications::NotificationRepository>,
}

#[async_trait]
pub trait NotificationService: Send + Sync {
	async fn create_notification(&self, notification: notifications::Notification)
	-> AppResult<()>;
}

impl NotificationServiceImpl {
	pub fn new(repository: Arc<dyn notifications::NotificationRepository>) -> Self {
		NotificationServiceImpl { repository }
	}
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
	async fn create_notification(
		&self,
		notification: notifications::Notification,
	) -> AppResult<()> {
		self.repository.create(notification).await.map(|_| ())
	}
}
