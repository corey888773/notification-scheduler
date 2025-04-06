use async_nats::jetstream::{Context};
use async_trait::async_trait;
use std::sync::Arc;
use async_nats::HeaderMap;
use async_nats::jetstream::stream::Config;
use crate::utils::{errors::AppError, types::AppResult};

#[async_trait]
pub trait Broker: Send + Sync {
	// Note: we add a key parameter to use for the deduplication header.
	async fn send_message(&self, topic: String, payload: String, key: String) -> AppResult<()>;
}

pub struct NatsImpl {
	client: async_nats::Client,
	js: Arc<Context>,
}

pub struct NatsOptions {
	pub nats_url: String,
}

impl NatsImpl {
	pub async fn new(opts: NatsOptions) -> Self {
		// Connect to the NATS server.
		let client = async_nats::connect(&opts.nats_url)
			.await
			.expect("Failed to connect to NATS");
		// Create a JetStream context.
		let js = async_nats::jetstream::new(client.clone());
		// Create a stream if it doesn't exist.
		let stream_config = Config{
			name: "notifications".to_string(),
			subjects: vec!["notifications.*".to_string()],
			retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
			duplicate_window: std::time::Duration::from_secs(60),
			max_age: std::time::Duration::from_secs(600),
			..Default::default()
		};
		if js.get_stream("notifications").await.is_ok() {
			println!("Stream already exists");

			// modify
			js.update_stream(stream_config).await.expect("Failed to update stream");

			return Self {
				client,
				js: Arc::new(js),
			};
		}

		js.create_stream(stream_config)
			.await
			.expect("Failed to create stream");

		Self {
			client,
			js: Arc::new(js),
		}
	}
}

#[async_trait]
impl Broker for NatsImpl {
	async fn send_message(&self, topic: String, payload: String, key: String) -> AppResult<()> {
		let mut headers = HeaderMap::new();
		headers.insert("Nats-Msg-Id", key);

		let topic = format!("notifications.{}", topic);
		let publish_ack = self
			.js
			.publish_with_headers(topic, headers, payload.into())
			.await
			.map_err(|e| AppError::ServiceError(format!("Failed to publish message: {}", e)))?;

		// Await the ack.
		let ack = publish_ack
			.await
			.map_err(|e| AppError::ServiceError(format!("Failed to ack message: {}", e)))?;

		if ack.stream.is_empty() {
			return Err(AppError::ServiceError("Failed to ack message".to_string()));
		}

		println!("Message published with ack: {:?}", ack);
		Ok(())
	}
}