use futures::StreamExt;
use log::error;
use tokio::time::{Duration, sleep};

use crate::handler::MessageHandler;

// Define a NATS consumer that uses JetStream.
pub struct NatsConsumer {
	consumer: async_nats::jetstream::consumer::PullConsumer,
}

pub struct NatsConsumerOptions {
	pub nats_url:       String,
	pub recipient_id:   String,
	pub channel:        String,
	pub filter_subject: String,
}

impl NatsConsumer {
	/// Connects to NATS, creates a JetStream context, and subscribes to the
	/// "email" subject.
	pub async fn new(opts: NatsConsumerOptions) -> Self {
		// Connect to the NATS server.
		let client = async_nats::connect(&opts.nats_url)
			.await
			.expect("Failed to connect to NATS");
		// Create a JetStream context.
		let js = async_nats::jetstream::new(client);
		// Subscribe to the "email" subject.
		let stream = js
			.get_stream(format!("notifications_{}", opts.channel,))
			.await
			.expect("Failed to get stream");
		let consumer_config = async_nats::jetstream::consumer::pull::Config {
			name: Some(opts.recipient_id.clone()),
			ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
			ack_wait: Duration::from_secs(60),
			filter_subject: opts.filter_subject.clone(),
			..Default::default()
		};
		let consumer = stream
			.create_consumer(consumer_config)
			.await
			.expect("Failed to create consumer");

		Self { consumer }
	}

	pub async fn start(&mut self, handler: Box<dyn MessageHandler>) {
		let mut messages_stream = self
			.consumer
			.messages()
			.await
			.expect("Failed to get messages stream");

		while let Some(message_result) = messages_stream.next().await {
			match message_result {
				Ok(msg) => match std::str::from_utf8(&msg.payload) {
					Ok(payload_str) => {
						handler.handle_message(payload_str);
						if let Err(e) = msg.ack().await {
							error!("Failed to ack message: {:?}", e);
						}
					}
					Err(e) => {
						error!("Received message with invalid UTF-8: {:?}", e);
					}
				},
				Err(e) => error!("Error receiving message: {:?}", e),
			}
			// Simulate processing time.
			sleep(Duration::from_secs(10)).await;
		}
	}
}
