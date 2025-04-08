use std::{sync::Arc, time::Duration};

use log::debug;

use crate::{
	app_state,
	crone::scheduler::{CronScheduler, CronSchedulerImpl},
	data::notifications::Priority,
};

pub(crate) mod scheduler;

pub fn build_notification_scheduler_future(
	app_state: Arc<app_state::AppState>,
	priority: Priority,
	duration: Duration,
) -> impl Future<Output = ()> + Send {
	let scheduler = CronSchedulerImpl::new(duration, move || {
		let notification_service = app_state.notification_service.clone();
		let priority = priority.clone();
		async move {
			let utc_now = chrono::Utc::now();
			debug!("{:?} priority scheduler running at: {}", priority, utc_now);
			notification_service
				.send_messages(priority, utc_now)
				.await
				.unwrap();
		}
	});

	async move { scheduler.start().await }
}
