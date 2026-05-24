//! # Benchmark Module for Cargo-Vendormod
//!
//! Performance benchmarking utilities for measuring workload processing.

use std::time::Instant;

/// Simple benchmark timing utility
pub struct BenchmarkTimer {
    name: String,
    start: Instant,
}

impl BenchmarkTimer {
    /// Create a new timer with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    /// Finish timing and return elapsed duration in milliseconds
    pub fn finish(self) -> u128 {
        let elapsed = self.start.elapsed().as_millis();
        eprintln!("⏱️  {}: {}ms", self.name, elapsed);
        elapsed
    }
}

/// Macro to benchmark a block of code
#[macro_export]
macro_rules! benchmark {
    ($name:expr, $block:block) => {{
        let timer = $crate::benchmark::BenchmarkTimer::new($name);
        let result = $block;
        timer.finish();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_benchmark_timer_creation() {
        let timer = BenchmarkTimer::new("test");
        assert_eq!(timer.name, "test");
    }

    #[test]
    fn test_benchmark_timer_finish() {
        let timer = BenchmarkTimer::new("timing_test");
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.finish();
        assert!(elapsed >= 10, "Timer should have recorded at least 10ms");
    }

    #[test]
    fn test_benchmark_macro() {
        let result = benchmark!("test_macro", {
            thread::sleep(Duration::from_millis(5));
            42
        });
        assert_eq!(result, 42);
    }
}