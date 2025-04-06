use std::sync::Arc;

use crate::{
	data,
	data::notifications::NotificationRepository,
	messaging,
	messaging::broker::NatsOptions,
	services,
	services::notifications::NotificationService,
};

pub struct AppStateOptions {
	pub mongo_url: String,
	pub nats_url:  String,
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
		let nats_broker: Arc<dyn messaging::broker::Broker> = Arc::new(
			messaging::broker::NatsImpl::new(NatsOptions {
				nats_url: opts.nats_url.clone(),
			})
			.await,
		);
		let notification_service: Arc<dyn NotificationService> =
			Arc::new(services::notifications::NotificationServiceImpl::new(
				notification_repository,
				nats_broker,
			));

		Arc::new(AppState {
			notification_service,
		})
	}
}
