// Benchmarks have relaxed clippy rules
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

//! MemoryEvent Broadcast Channel Benchmarks
//!
//! Benchmarks for the tokio::sync::broadcast channel used for lifecycle events.
//! Part of WG-103: MemoryEvent broadcast channel.
//!
//! Run with: `cargo bench --bench memory_event_benchmark`

use criterion::{Criterion, criterion_group, criterion_main};
use do_memory_core::types::{DEFAULT_EVENT_CHANNEL_CAPACITY, MemoryEvent};
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

/// Create a tokio runtime for the benchmarks
fn rt() -> &'static Runtime {
    static RUNTIME: once_cell::sync::Lazy<Runtime> =
        once_cell::sync::Lazy::new(|| Runtime::new().expect("Failed to create runtime"));
    &RUNTIME
}

/// Create a test MemoryEvent
fn create_test_event(i: u64) -> MemoryEvent {
    MemoryEvent::EpisodeCreated {
        id: format!("episode-{}", i),
        task: format!("Benchmark task {}", i),
        timestamp: i,
    }
}

/// Benchmark: Single sender, single receiver
fn bench_broadcast_single_receiver(c: &mut Criterion) {
    let mut group = c.benchmark_group("broadcast_single_receiver");

    group.bench_function("send_1000_events", |b| {
        b.iter(|| {
            let (tx, mut rx) = broadcast::channel::<MemoryEvent>(DEFAULT_EVENT_CHANNEL_CAPACITY);

            rt().block_on(async {
                // Send events
                for i in 0..1000 {
                    tx.send(create_test_event(i)).unwrap();
                }

                // Receive all events
                let mut count = 0;
                while rx.try_recv().is_ok() {
                    count += 1;
                }
                black_box(count);
            });
        })
    });

    group.finish();
}

/// Benchmark: Single sender, multiple receivers (fan-out)
#[allow(clippy::excessive_nesting)]
fn bench_broadcast_fan_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("broadcast_fan_out");

    for num_receivers in [1, 5, 10, 50] {
        group.bench_function(format!("fan_out_{}_receivers", num_receivers), |b| {
            b.iter(|| {
                let (tx, _) = broadcast::channel::<MemoryEvent>(DEFAULT_EVENT_CHANNEL_CAPACITY);

                rt().block_on(async {
                    // Create multiple receivers
                    let receivers: Vec<_> = (0..num_receivers).map(|_| tx.subscribe()).collect();

                    // Send events
                    for i in 0..100 {
                        tx.send(create_test_event(i)).unwrap();
                    }

                    // All receivers should get all events
                    for mut rx in receivers {
                        let mut count = 0;
                        while rx.try_recv().is_ok() {
                            count += 1;
                        }
                        black_box(count);
                    }
                });
            })
        });
    }

    group.finish();
}

/// Benchmark: Subscribe overhead
fn bench_subscribe_overhead(c: &mut Criterion) {
    let (tx, _) = broadcast::channel::<MemoryEvent>(DEFAULT_EVENT_CHANNEL_CAPACITY);

    c.bench_function("subscribe", |b| b.iter(|| black_box(tx.subscribe())));
}

/// Benchmark: Send overhead
fn bench_send_overhead(c: &mut Criterion) {
    let (tx, _) = broadcast::channel::<MemoryEvent>(DEFAULT_EVENT_CHANNEL_CAPACITY);
    let event = create_test_event(0);

    c.bench_function("send_event", |b| {
        b.iter(|| black_box(tx.send(event.clone())))
    });
}

/// Benchmark: Full lifecycle simulation
fn bench_lifecycle_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle_simulation");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("episode_lifecycle_events", |b| {
        b.iter(|| {
            let (tx, mut rx) = broadcast::channel::<MemoryEvent>(DEFAULT_EVENT_CHANNEL_CAPACITY);

            rt().block_on(async {
                // Simulate episode lifecycle
                let episode_id = "benchmark-episode";

                // EpisodeCreated
                tx.send(MemoryEvent::EpisodeCreated {
                    id: episode_id.to_string(),
                    task: "Benchmark task".to_string(),
                    timestamp: 0,
                })
                .unwrap();

                // EpisodeCompleted
                tx.send(MemoryEvent::EpisodeCompleted {
                    id: episode_id.to_string(),
                    reward: 0.85,
                    timestamp: 1,
                })
                .unwrap();

                // Receive events
                let mut count = 0;
                while rx.try_recv().is_ok() {
                    count += 1;
                }
                black_box(count);
            });
        })
    });

    group.finish();
}

/// Benchmark: Channel capacity stress test
#[allow(clippy::excessive_nesting)]
fn bench_capacity_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity_stress");

    for capacity in [64, 256, 1024, 4096] {
        group.bench_function(format!("capacity_{}", capacity), |b| {
            b.iter(|| {
                let (tx, mut rx) = broadcast::channel::<MemoryEvent>(capacity);

                rt().block_on(async {
                    // Fill the channel
                    for i in 0..capacity {
                        let _ = tx.send(create_test_event(i as u64));
                    }

                    // Drain
                    let mut count = 0;
                    while rx.try_recv().is_ok() {
                        count += 1;
                    }
                    black_box(count);
                });
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_broadcast_single_receiver,
    bench_broadcast_fan_out,
    bench_subscribe_overhead,
    bench_send_overhead,
    bench_lifecycle_simulation,
    bench_capacity_stress,
);
criterion_main!(benches);
