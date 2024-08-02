#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_increment_counter() {
        let metrics = Metrics::instance();

        metrics.increment_counter("test_counter");
        metrics.increment_counter("test_counter");

        let counters = metrics.get_counters();
        assert_eq!(counters.get("test_counter"), Some(&2));
    }

    #[test]
    fn test_set_gauge() {
        let metrics = Metrics::instance();

        metrics.set_gauge("test_gauge", 5);
        metrics.set_gauge("test_gauge", 10);

        let gauges = metrics.get_gauges();
        assert_eq!(gauges.get("test_gauge"), Some(&10));
    }

    #[test]
    fn test_update_histogram() {
        let metrics = Metrics::instance();

        metrics.update_histogram("test_histogram", 5.0);
        metrics.update_histogram("test_histogram", 15.0);

        let histograms = metrics.get_histograms();
        let (sum, count) = histograms.get("test_histogram").unwrap();
        assert_eq!(*sum, 20.0);
        assert_eq!(*count, 2);
    }

    #[test]
    fn test_get_counters() {
        let metrics = Metrics::instance();
        metrics.increment_counter("counter1");
        metrics.increment_counter("counter2");
        metrics.increment_counter("counter1");

        let counters = metrics.get_counters();
        let mut expected = HashMap::new();
        expected.insert("counter1".to_string(), 2);
        expected.insert("counter2".to_string(), 1);

        assert_eq!(counters, expected);
    }

    #[test]
    fn test_get_gauges() {
        let metrics = Metrics::instance();
        metrics.set_gauge("gauge1", 5);
        metrics.set_gauge("gauge2", 10);
        metrics.set_gauge("gauge1", 15);

        let gauges = metrics.get_gauges();
        let mut expected = HashMap::new();
        expected.insert("gauge1".to_string(), 15);
        expected.insert("gauge2".to_string(), 10);

        assert_eq!(gauges, expected);
    }

    #[test]
    fn test_get_histograms() {
        let metrics = Metrics::instance();
        metrics.update_histogram("hist1", 1.0);
        metrics.update_histogram("hist1", 2.0);
        metrics.update_histogram("hist2", 3.0);

        let histograms = metrics.get_histograms();
        let mut expected = HashMap::new();
        expected.insert("hist1".to_string(), (3.0, 2));
        expected.insert("hist2".to_string(), (3.0, 1));

        assert_eq!(histograms, expected);
    }
}
