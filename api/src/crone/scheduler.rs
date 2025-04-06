use std::time::Duration;

use async_trait::async_trait;
use common::tokio::time;

#[async_trait]
pub trait CronScheduler {
	async fn start(&self);
}

// Our scheduler struct holds the interval and a task function.
pub struct CronSchedulerImpl<F, Fut>
where
	F: Fn() -> Fut + Send + Sync + 'static,
	Fut: Future<Output = ()> + Send,
{
	interval: Duration,
	task:     F,
}

impl<F, Fut> CronSchedulerImpl<F, Fut>
where
	F: Fn() -> Fut + Send + Sync + 'static,
	Fut: Future<Output = ()> + Send,
{
	pub fn new(interval: Duration, task: F) -> Self {
		Self { interval, task }
	}
}

#[async_trait]
impl<F, Fut> CronScheduler for CronSchedulerImpl<F, Fut>
where
	F: Fn() -> Fut + Send + Sync + 'static,
	Fut: Future<Output = ()> + Send,
{
	async fn start(&self) {
		let mut ticker = time::interval(self.interval);
		loop {
			// Wait for the next tick.
			ticker.tick().await;
			// Execute the task.
			(self.task)().await;
		}
	}
}
