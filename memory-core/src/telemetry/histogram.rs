use crate::telemetry::HistogramStats;
use std::sync::Mutex;
use std::time::Duration;

/// Histogram metric for tracking distributions of values
///
/// Histograms collect samples and compute percentiles for
/// operation durations, request sizes, etc.
#[derive(Debug)]
pub struct Histogram {
    /// Collected samples in milliseconds
    samples: Mutex<Vec<f64>>,
    /// Number of buckets for power-of-two histogram
    buckets: usize,
}

impl Histogram {
    /// Create a new histogram
    ///
    /// # Arguments
    ///
    /// * `buckets` - Number of power-of-two buckets (e.g., 16 for buckets up to 65536ms)
    #[must_use]
    pub fn new(buckets: usize) -> Self {
        Self {
            samples: Mutex::new(Vec::new()),
            buckets,
        }
    }

    /// Record a duration observation
    pub fn observe(&self, duration: Duration) {
        let millis = duration.as_secs_f64() * 1000.0;
        self.observe_millis(millis);
    }

    /// Record a value in milliseconds
    pub fn observe_millis(&self, millis: f64) {
        let mut samples = self.samples.lock().unwrap();
        samples.push(millis);
    }

    /// Get histogram statistics
    #[must_use]
    pub fn get_stats(&self) -> HistogramStats {
        let samples = self.samples.lock().unwrap();

        if samples.is_empty() {
            return HistogramStats {
                count: 0,
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }

        let count = samples.len() as u64;
        let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = samples.iter().sum();
        let avg = sum / count as f64;

        // Sort for percentile calculation
        let mut sorted = samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = percentile(&sorted, 0.50);
        let p95 = percentile(&sorted, 0.95);
        let p99 = percentile(&sorted, 0.99);

        HistogramStats {
            count,
            min,
            max,
            avg,
            p50,
            p95,
            p99,
        }
    }

    /// Reset the histogram
    pub fn reset(&self) {
        let mut samples = self.samples.lock().unwrap();
        samples.clear();
    }
}

/// Calculate percentile from sorted data
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    let idx = ((sorted.len() as f64 - 1.0) * p) as usize;
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe() {
        let histogram = Histogram::new(16);

        histogram.observe_millis(10.0);
        histogram.observe_millis(20.0);
        histogram.observe_millis(30.0);

        let stats = histogram.get_stats();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
        assert_eq!(stats.avg, 20.0);
    }

    #[test]
    fn test_percentiles() {
        let histogram = Histogram::new(16);

        // Create 100 samples from 1 to 100
        for i in 1..=100 {
            histogram.observe_millis(i as f64);
        }

        let stats = histogram.get_stats();
        assert_eq!(stats.count, 100);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 100.0);

        // Check percentiles are reasonable
        assert!(stats.p50 >= 48.0 && stats.p50 <= 52.0);
        assert!(stats.p95 >= 93.0 && stats.p95 <= 97.0);
        assert!(stats.p99 >= 98.0 && stats.p99 <= 100.0);
    }

    #[test]
    fn test_reset() {
        let histogram = Histogram::new(16);

        histogram.observe_millis(10.0);
        histogram.reset();

        let stats = histogram.get_stats();
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_duration() {
        let histogram = Histogram::new(16);

        histogram.observe(Duration::from_millis(123));
        histogram.observe(Duration::from_nanos(456_000)); // 0.456ms

        let stats = histogram.get_stats();
        assert_eq!(stats.count, 2);
        assert!((stats.min - 0.456).abs() < 0.001);
        assert!((stats.max - 123.0).abs() < 0.1);
    }
}
