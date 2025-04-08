pub mod std_out;

pub trait MessageHandler {
	fn handle_message(&self, message: &str);
}
