use std::sync::Arc;

use async_nats::{
	HeaderMap,
	jetstream::{Context, stream::Config},
};
use async_trait::async_trait;

use crate::utils::{errors::AppError, types::AppResult};

#[async_trait]
pub trait Broker: Send + Sync {
	async fn send_message(
		&self,
		channel: String,
		recipient: String,
		payload: String,
		dedup_key: String,
	) -> AppResult<()>;
}

pub struct NatsImpl {
	client: async_nats::Client,
	js:     Arc<Context>,
}

pub struct NatsOptions {
	pub nats_url: String,
}

impl NatsImpl {
	pub async fn new(opts: NatsOptions) -> Self {
		let client = async_nats::connect(&opts.nats_url)
			.await
			.expect("Failed to connect to NATS");
		let js = async_nats::jetstream::new(client.clone());
		setup_streams(
			&js,
			vec![
				"notifications_email".to_string(),
				"notifications_push".to_string(),
			],
		)
		.await
		.expect("Failed to setup streams");
		Self {
			client,
			js: Arc::new(js),
		}
	}
}

#[async_trait]
impl Broker for NatsImpl {
	async fn send_message(
		&self,
		channel: String,
		recipient: String,
		payload: String,
		dedup_key: String,
	) -> AppResult<()> {
		let mut headers = HeaderMap::new();
		headers.insert("Nats-Msg-Id", dedup_key);

		let topic = format!("notifications_{}.{}", channel, recipient);
		let publish_ack = self
			.js
			.publish_with_headers(topic, headers, payload.into())
			.await
			.map_err(|e| AppError::ServiceError(format!("Failed to publish message: {}", e)))?;

		let ack = publish_ack
			.await
			.map_err(|e| AppError::ServiceError(format!("Failed to ack message: {}", e)))?;

		if ack.stream.is_empty() {
			return Err(AppError::ServiceError("Failed to ack message".to_string()));
		}

		Ok(())
	}
}

async fn setup_streams(js: &Context, streams: Vec<String>) -> AppResult<()> {
	for stream in streams {
		let stream_config = Config {
			name: stream.clone(),
			subjects: vec![format!("{}.*", stream)],
			retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
			duplicate_window: std::time::Duration::from_secs(60),
			max_age: std::time::Duration::from_secs(60 * 60 * 24),
			..Default::default()
		};

		js.get_or_create_stream(stream_config)
			.await
			.map_err(|e| AppError::ServiceError(format!("Failed to create stream: {}", e)))?;
	}
	Ok(())
}
