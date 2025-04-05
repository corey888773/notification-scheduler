use std::sync::Arc;

use crate::{
	data,
	data::notifications::NotificationRepository,
	messaging,
	messaging::broker::KafkaOptions,
	services,
	services::notifications::NotificationService,
};

pub struct AppStateOptions {
	pub mongo_url: String,
	pub kafka_url: String,
}

pub struct AppState {
	pub notification_service: Arc<dyn NotificationService>,
}

impl AppState {
	pub async fn new(opts: AppStateOptions) -> Arc<AppState> {
		let db = data::db::DbContext::new(opts.mongo_url.as_ref())
			.await
			.expect("Failed to set up database");

		let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
			data::notifications::NotificationRepositoryImpl::new(db.notifications_collection),
		);
		let kafka_broker: Arc<dyn messaging::broker::Broker> =
			Arc::new(messaging::broker::KafkaImpl::new(KafkaOptions {
				kafka_url: opts.kafka_url,
			}));
		let notification_service: Arc<dyn NotificationService> =
			Arc::new(services::notifications::NotificationServiceImpl::new(
				notification_repository,
				kafka_broker,
			));

		Arc::new(AppState {
			notification_service,
		})
	}
}
