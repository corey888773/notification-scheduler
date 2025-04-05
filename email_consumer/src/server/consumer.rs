use futures::StreamExt;
use rdkafka::{
	Message,
	consumer::{CommitMode, Consumer, StreamConsumer},
};

pub struct KafkaConsumer {
	message_consumer: StreamConsumer,
}

impl KafkaConsumer {
	pub fn new(kafka_url: String) -> Self {
		let message_consumer: StreamConsumer = rdkafka::ClientConfig::new()
			.set("bootstrap.servers", &kafka_url)
			.set("group.id", "email_consumer")
			.set("enable.auto.commit", "false")
			.create()
			.expect("Consumer creation failed");

		message_consumer
			.subscribe(&["email"])
			.expect("Subscription failed");
		Self { message_consumer }
	}

	pub async fn start(&self) {
		let mut message_stream = self.message_consumer.stream();
		while let Some(message_result) = message_stream.next().await {
			match message_result {
				Ok(message) => {
					if let Some(Ok(payload)) = message.payload_view::<str>() {
						println!("Received: {}", payload);
						self.message_consumer
							.commit_message(&message, CommitMode::Async)
							.unwrap();
					} else {
						eprintln!("Received message with empty payload");
					}
				}
				Err(e) => eprintln!("Error receiving message: {:?}", e),
			}
		}
	}
}
