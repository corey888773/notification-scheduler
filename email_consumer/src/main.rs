mod server;

#[tokio::main]
async fn main() {
	dotenvy::from_filename("email_consumer/src/app.env").ok();
	let kafka_url = "localhost:9092".to_string();
	let kafka_consumer = server::consumer::KafkaConsumer::new(kafka_url);

	println!("Kafka consumer started");
	kafka_consumer.start().await;
}
