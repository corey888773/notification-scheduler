use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use axum_prometheus::metrics::{Counter, Gauge, Histogram, Key, Level, Metadata, Recorder};
use axum_prometheus::metrics_exporter_prometheus::PrometheusRecorder;

/// Generic metrics container with type-specific maps
pub struct Metrics {
    counters: Mutex<HashMap<String, Counter>>,
    gauges: Mutex<HashMap<String, Gauge>>,
    histograms: Mutex<HashMap<String, Histogram>>,
    prometheus_recorder: Arc<PrometheusRecorder>,
}

impl Metrics {
    pub fn new(prometheus_recorder: Arc<PrometheusRecorder>) -> Self {
        Metrics {
            counters: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
            histograms: Mutex::new(HashMap::new()),
            prometheus_recorder,
        }
    }

    pub fn register_counter(&self, name: &str, help: &str) -> Result<(), String> {
        let key = Key::from_name(name.to_string());
        let metadata = Metadata::new("app", Level::TRACE, Some("email_consumer")); // actually it is not used
        let counter = self.prometheus_recorder.register_counter(&key, &metadata);
        self.counters.lock().unwrap().insert(name.to_string(), counter);
        Ok(())
    }

    pub fn register_gauge(&self, name: &str, help: &str) -> Result<(), String> {
        let key = Key::from_name(name.to_string());
        let metadata = Metadata::new("app", Level::TRACE, Some("email_consumer")); // actually it is not used
        let gauge = self.prometheus_recorder.register_gauge(&key, &metadata);
        self.gauges.lock().unwrap().insert(name.to_string(), gauge);
        Ok(())
    }

    pub fn register_histogram(&self, name: &str, help: &str, buckets: Vec<f64>) -> Result<(), String> {
        let key = Key::from_name(name.to_string());
        let metadata = Metadata::new("app", Level::TRACE, Some("email_consumer")); // actually it is not used
        let histogram = self.prometheus_recorder.register_histogram(&key, &metadata);
        self.histograms.lock().unwrap().insert(name.to_string(), histogram);
        Ok(())
    }

    pub fn get_counter(&self, name: &str) -> Option<Counter> {
        self.counters.lock().unwrap().get(name).cloned()
    }

    pub fn get_gauge(&self, name: &str) -> Option<Gauge> {
        self.gauges.lock().unwrap().get(name).cloned()
    }

    pub fn get_histogram(&self, name: &str) -> Option<Histogram> {
        self.histograms.lock().unwrap().get(name).cloned()
    }
}