use std::sync::Arc;

use futures::StreamExt;
use tokio::time::{Duration, sleep};

use crate::{app_state::AppState, metrics::metrics::EMAIL_CONSUMER_CONSUMED_MESSAGES};

// Define a NATS consumer that uses JetStream.
pub struct NatsConsumer {
	consumer:  async_nats::jetstream::consumer::PullConsumer,
	app_state: Arc<AppState>,
}

impl NatsConsumer {
	/// Connects to NATS, creates a JetStream context, and subscribes to the
	/// "email" subject.
	pub async fn new(nats_url: String, app_state: Arc<AppState>) -> Self {
		// Connect to the NATS server.
		let client = async_nats::connect(&nats_url)
			.await
			.expect("Failed to connect to NATS");
		// Create a JetStream context.
		let js = async_nats::jetstream::new(client);
		// Subscribe to the "email" subject.
		let stream = js
			.get_stream("notifications")
			.await
			.expect("Failed to get stream");
		let consumer_config = async_nats::jetstream::consumer::pull::Config {
			name: Some("email_consumer".to_string()),
			ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
			ack_wait: Duration::from_secs(60),
			filter_subject: "notifications.email".to_string(),
			..Default::default()
		};
		let consumer = stream
			.create_consumer(consumer_config)
			.await
			.expect("Failed to create consumer");

		Self {
			consumer,
			app_state,
		}
	}

	/// Start consuming messages from the "email" subject.
	/// For each message, print the payload, update a metric, and ack the
	/// message.
	pub async fn start(&mut self) {
		let mut messages_stream = self
			.consumer
			.messages()
			.await
			.expect("Failed to get messages stream");
		while let Some(message_result) = messages_stream.next().await {
			match message_result {
				Ok(msg) => {
					// Try to interpret the payload as a UTF-8 string.
					match std::str::from_utf8(&msg.payload) {
						Ok(payload_str) => {
							println!("Received: {}", payload_str);
							if let Some(metric) = self
								.app_state
								.metrics
								.get_counter(EMAIL_CONSUMER_CONSUMED_MESSAGES)
							{
								metric.increment(1);
							}
							// Acknowledge the message.
							if let Err(e) = msg.ack().await {
								eprintln!("Failed to ack message: {:?}", e);
							}
						}
						Err(e) => {
							eprintln!("Received message with invalid UTF-8: {:?}", e);
						}
					}
				}
				Err(e) => eprintln!("Error receiving message: {:?}", e),
			}
			// Simulate processing time.
			sleep(Duration::from_secs(10)).await;
		}
	}
}
