use std::time::Duration;

use async_trait::async_trait;
use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::utils::{errors::AppError, types::AppResult};

#[async_trait]
pub trait Broker: Send + Sync {
	async fn send_message(&self, topic: &str, payload: &str, key: &str) -> AppResult<()>;
}

pub struct KafkaOptions {
	pub kafka_url: String,
}

pub struct KafkaImpl {
	producer: FutureProducer,
}

impl KafkaImpl {
	pub fn new(opts: KafkaOptions) -> Self {
		let producer = rdkafka::ClientConfig::new()
			.set("bootstrap.servers", opts.kafka_url)
			.create()
			.expect("Producer creation failed");

		Self { producer }
	}
}

#[async_trait]
impl Broker for KafkaImpl {
	async fn send_message(&self, topic: &str, payload: &str, key: &str) -> AppResult<()> {
		let record = FutureRecord::to(topic).payload(payload).key(key);
		let timeout = Duration::from_secs(0);

		self.producer
			.send(record, timeout)
			.await
			.map_err(|e| AppError::ServiceError(e.0.to_string()))?;

		Ok(())
	}
}
