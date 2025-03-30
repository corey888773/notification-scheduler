use rdkafka::producer::FutureProducer;

pub trait Broker {}

pub struct KafkaOptions {
	kafka_url: String,
	topics:    Vec<String>,
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

impl Broker for KafkaImpl {}
