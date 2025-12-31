use std::sync::Mutex;

/// Gauge metric for point-in-time values
///
/// Gauges track values that can go up or down, such as
/// cache size or connection count.
#[derive(Debug)]
pub struct Gauge {
    /// Inner value protected by mutex
    value: Mutex<f64>,
}

impl Gauge {
    /// Create a new gauge with value 0.0
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: Mutex::new(0.0),
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: f64) {
        let mut val = self.value.lock().unwrap();
        *val = value;
    }

    /// Add a value to the current gauge value
    pub fn add(&self, delta: f64) {
        let mut val = self.value.lock().unwrap();
        *val += delta;
    }

    /// Subtract a value from the current gauge value
    pub fn sub(&self, delta: f64) {
        self.add(-delta);
    }

    /// Get the current gauge value
    #[must_use]
    pub fn get(&self) -> f64 {
        *self.value.lock().unwrap()
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let gauge = Gauge::new();
        gauge.set(42.0);

        assert_eq!(gauge.get(), 42.0);
    }

    #[test]
    fn test_add() {
        let gauge = Gauge::new();
        gauge.set(10.0);
        gauge.add(5.0);

        assert_eq!(gauge.get(), 15.0);
    }

    #[test]
    fn test_sub() {
        let gauge = Gauge::new();
        gauge.set(10.0);
        gauge.sub(3.0);

        assert_eq!(gauge.get(), 7.0);
    }

    #[test]
    fn test_default() {
        let gauge = Gauge::default();
        assert_eq!(gauge.get(), 0.0);
    }
}
