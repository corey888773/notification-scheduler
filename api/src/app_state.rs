use crate::services::notifications::NotificationService;
use std::sync::Arc;

pub struct AppState {
	pub notification_service: Arc<dyn NotificationService>,
}

impl AppState {
	pub fn new(notification_service: Arc<dyn NotificationService>) -> Arc<AppState> {
		Arc::new(AppState {
			notification_service,
		})
	}
}
