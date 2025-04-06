use std::sync::Arc;
use common::monitoring::metrics::Metrics;

pub struct AppState {
    pub metrics: Arc<Metrics>,
}