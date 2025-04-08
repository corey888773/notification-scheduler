use std::sync::Arc;

use log::info;

use crate::{
	app_state::AppState,
	handler::MessageHandler,
	metrics::metrics::EMAIL_CONSUMER_CONSUMED_MESSAGES,
};

pub struct StdOutHandlerImpl {
	pub app_state: Arc<AppState>,
}

impl MessageHandler for StdOutHandlerImpl {
	fn handle_message(&self, message: &str) {
		info!("Received: {}", message);
		if let Some(metric) = self
			.app_state
			.metrics
			.get_counter(EMAIL_CONSUMER_CONSUMED_MESSAGES)
		{
			metric.increment(1);
		}
	}
}
