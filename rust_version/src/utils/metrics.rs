use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct Metrics {
    counters: RwLock<HashMap<String, Arc<AtomicI64>>>,
    gauges: RwLock<HashMap<String, Arc<AtomicUsize>>>,
    histograms: RwLock<HashMap<String, Arc<(AtomicUsize, AtomicUsize)>>>,
}

impl Metrics {
    pub fn instance() -> Arc<Metrics> {
        static INSTANCE: std::sync::OnceLock<Arc<Metrics>> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(Metrics::new())).clone()
    }

    fn new() -> Self {
        Metrics {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    pub fn increment_counter(&self, name: &str) {
        let counters = self.counters.read();
        let counter = counters.get(name).cloned().unwrap_or_else(|| {
            let new_counter = Arc::new(AtomicI64::new(0));
            drop(counters);
            self.counters.write().entry(name.to_string()).or_insert_with(|| new_counter.clone());
            new_counter
        });
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_gauge(&self, name: &str, value: usize) {
        let gauges = self.gauges.read();
        let gauge = gauges.get(name).cloned().unwrap_or_else(|| {
            let new_gauge = Arc::new(AtomicUsize::new(0));
            drop(gauges);
            self.gauges.write().entry(name.to_string()).or_insert_with(|| new_gauge.clone());
            new_gauge
        });
        gauge.store(value, Ordering::Relaxed);
    }

    pub fn update_histogram(&self, name: &str, value: f64) {
        let histograms = self.histograms.read();
        let histogram = histograms.get(name).cloned().unwrap_or_else(|| {
            let new_histogram = Arc::new((AtomicUsize::new(0), AtomicUsize::new(0)));
            drop(histograms);
            self.histograms.write().entry(name.to_string()).or_insert_with(|| new_histogram.clone());
            new_histogram
        });

        let (sum, count) = &*histogram;
        sum.fetch_add(value as usize, Ordering::Relaxed);
        count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_counters(&self) -> HashMap<String, i64> {
        self.counters
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    pub fn get_gauges(&self) -> HashMap<String, usize> {
        self.gauges
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    pub fn get_histograms(&self) -> HashMap<String, (f64, usize)> {
        self.histograms
            .read()
            .iter()
            .map(|(k, v)| {
                let sum = v.0.load(Ordering::Relaxed) as f64;
                let count = v.1.load(Ordering::Relaxed);
                (k.clone(), (sum, count))
            })
            .collect()
    }
}