//! Configuration and metrics for adaptive connection pool

use memory_core::Result;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AdaptivePoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
    pub scale_up_increment: u32,
    pub scale_down_decrement: u32,
    pub check_interval: Duration,
}

impl Default for AdaptivePoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 50,
            scale_up_threshold: 0.7,
            scale_down_threshold: 0.3,
            scale_up_cooldown: Duration::from_secs(10),
            scale_down_cooldown: Duration::from_secs(30),
            scale_up_increment: 5,
            scale_down_decrement: 5,
            check_interval: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Default)]
pub struct AdaptivePoolMetrics {
    pub utilization_percent: f64,
    pub active_connections: u32,
    pub max_connections: u32,
    pub scale_up_count: u32,
    pub scale_down_count: u32,
    pub avg_wait_time_us: u64,
    pub total_acquired: u64,
    pub total_released: u64,
}

#[derive(Debug)]
pub struct AdaptiveMetrics {
    pub utilization_percent: AtomicU64,
    pub active_connections: AtomicU32,
    pub max_connections: AtomicU32,
    pub scale_up_count: AtomicU32,
    pub scale_down_count: AtomicU32,
    pub avg_wait_time_us: AtomicU64,
    pub total_acquired: AtomicU64,
    pub total_released: AtomicU64,
    pub wait_time_total_us: AtomicU64,
    pub wait_count: AtomicU64,
    pub last_scale_up: AtomicU64,
    pub last_scale_down: AtomicU64,
}

impl Default for AdaptiveMetrics {
    fn default() -> Self {
        Self {
            utilization_percent: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
            max_connections: AtomicU32::new(0),
            scale_up_count: AtomicU32::new(0),
            scale_down_count: AtomicU32::new(0),
            avg_wait_time_us: AtomicU64::new(0),
            total_acquired: AtomicU64::new(0),
            total_released: AtomicU64::new(0),
            wait_time_total_us: AtomicU64::new(0),
            wait_count: AtomicU64::new(0),
            last_scale_up: AtomicU64::new(0),
            last_scale_down: AtomicU64::new(0),
        }
    }
}
