//! Memory pressure testing benchmarks
//!
//! Tests memory usage patterns under various loads, monitoring:
//! - Memory consumption over time
//! - Garbage collection pressure
//! - Memory fragmentation
//! - Peak memory usage under concurrent load

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_benches::TokioExecutor;
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_core::types::{TaskOutcome, TaskType};
use std::sync::Arc;
use sysinfo::{Pid, System};
use tokio::time::{sleep, Duration};

/// Memory pressure test scenarios
#[derive(Debug, Clone)]
enum MemoryScenario {
    /// Steady state: constant memory usage
    SteadyState,
    /// Burst load: periodic high memory usage spikes
    BurstLoad,
    /// Memory leak simulation: gradually increasing memory usage
    GradualGrowth,
    /// High concurrency: many concurrent operations
    HighConcurrency,
}

impl MemoryScenario {
    fn name(&self) -> &'static str {
        match self {
            Self::SteadyState => "steady_state",
            Self::BurstLoad => "burst_load",
            Self::GradualGrowth => "gradual_growth",
            Self::HighConcurrency => "high_concurrency",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::SteadyState => "Constant memory usage with steady operation rate",
            Self::BurstLoad => "Periodic spikes in memory usage",
            Self::GradualGrowth => "Gradually increasing memory usage over time",
            Self::HighConcurrency => "High concurrent operations with memory pressure",
        }
    }
}

struct MemoryMonitor {
    system: System,
    pid: Pid,
    measurements: Vec<MemoryMeasurement>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MemoryMeasurement {
    timestamp: std::time::Instant,
    memory_bytes: u64,
    memory_mb: f32,
    cpu_usage: f32,
}

impl MemoryMonitor {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = std::process::id();
        let pid = Pid::from(pid as usize);

        Self {
            system,
            pid,
            measurements: Vec::new(),
        }
    }

    fn record_measurement(&mut self) {
        self.system.refresh_all();

        if let Some(process) = self.system.processes().get(&self.pid) {
            let measurement = MemoryMeasurement {
                timestamp: std::time::Instant::now(),
                memory_bytes: process.memory(),
                memory_mb: process.memory() as f32 / 1024.0 / 1024.0,
                cpu_usage: process.cpu_usage(),
            };

            self.measurements.push(measurement);
        }
    }

    fn get_peak_memory_mb(&self) -> f32 {
        self.measurements
            .iter()
            .map(|m| m.memory_mb)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn get_average_memory_mb(&self) -> f32 {
        if self.measurements.is_empty() {
            0.0
        } else {
            self.measurements.iter().map(|m| m.memory_mb).sum::<f32>()
                / self.measurements.len() as f32
        }
    }

    fn get_memory_variance(&self) -> f32 {
        if self.measurements.len() < 2 {
            0.0
        } else {
            let mean = self.get_average_memory_mb();
            let variance = self
                .measurements
                .iter()
                .map(|m| (m.memory_mb - mean).powi(2))
                .sum::<f32>()
                / (self.measurements.len() - 1) as f32;
            variance.sqrt() // Standard deviation
        }
    }
}

async fn run_memory_pressure_scenario(
    memory: Arc<memory_core::memory::SelfLearningMemory>,
    scenario: MemoryScenario,
    duration_secs: u64,
) {
    let context = create_benchmark_context();
    let start_time = std::time::Instant::now();

    match scenario {
        MemoryScenario::SteadyState => {
            // Steady stream of operations
            while start_time.elapsed().as_secs() < duration_secs {
                let episode_id = memory
                    .start_episode(
                        generate_episode_description(rand::random::<usize>() % 1000),
                        context.clone(),
                        TaskType::CodeGeneration,
                    )
                    .await;

                let steps = generate_execution_steps(2);
                for step in steps {
                    memory.log_step(episode_id, step).await;
                }

                memory
                    .complete_episode(
                        episode_id,
                        TaskOutcome::Success {
                            verdict: "Steady state operation".to_string(),
                            artifacts: vec![],
                        },
                    )
                    .await
                    .expect("Failed to complete episode");

                // Small delay to prevent overwhelming
                sleep(Duration::from_millis(10)).await;
            }
        }

        MemoryScenario::BurstLoad => {
            while start_time.elapsed().as_secs() < duration_secs {
                // Burst of operations
                let mut handles = vec![];
                for _ in 0..10 {
                    let memory = memory.clone();
                    let context = context.clone();

                    let handle = tokio::spawn(async move {
                        #[allow(clippy::excessive_nesting)]
                        {
                            let episode_id = memory
                                .start_episode(
                                    generate_episode_description(rand::random::<usize>() % 100),
                                    context,
                                    TaskType::CodeGeneration,
                                )
                                .await;

                            let steps = generate_execution_steps(3);
                            for step in steps {
                                memory.log_step(episode_id, step).await;
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: "Burst operation".to_string(),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");
                        }
                    });

                    handles.push(handle);
                }

                // Wait for burst to complete
                futures::future::join_all(handles).await;

                // Rest period
                sleep(Duration::from_millis(500)).await;
            }
        }

        MemoryScenario::GradualGrowth => {
            let mut episode_count = 0;
            while start_time.elapsed().as_secs() < duration_secs {
                // Increasing batch sizes over time
                let batch_size = 1 + (episode_count / 10).min(20);

                let mut handles = vec![];
                for _ in 0..batch_size {
                    let memory = memory.clone();
                    let context = context.clone();

                    let handle = tokio::spawn(async move {
                        #[allow(clippy::excessive_nesting)]
                        {
                            let episode_id = memory
                                .start_episode(
                                    format!("Gradual growth episode {}", rand::random::<u32>()),
                                    context,
                                    TaskType::CodeGeneration,
                                )
                                .await;

                            let steps = generate_execution_steps(2 + (rand::random::<usize>() % 3));
                            for step in steps {
                                memory.log_step(episode_id, step).await;
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: "Gradual growth operation".to_string(),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");
                        }
                    });

                    handles.push(handle);
                }

                futures::future::join_all(handles).await;
                episode_count += batch_size;

                sleep(Duration::from_millis(100)).await;
            }
        }

        MemoryScenario::HighConcurrency => {
            // High concurrent operations
            let mut handles = vec![];

            for i in 0..100 {
                let memory = memory.clone();
                let context = context.clone();

                let handle = tokio::spawn(async move {
                    #[allow(clippy::excessive_nesting)]
                    {
                        for j in 0..10 {
                            let episode_id = memory
                                .start_episode(
                                    format!("Concurrent episode {}:{}", i, j),
                                    context.clone(),
                                    TaskType::CodeGeneration,
                                )
                                .await;

                            let steps = generate_execution_steps(2);
                            for step in steps {
                                memory.log_step(episode_id, step).await;
                            }

                            memory
                                .complete_episode(
                                    episode_id,
                                    TaskOutcome::Success {
                                        verdict: format!("Concurrent operation {}:{}", i, j),
                                        artifacts: vec![],
                                    },
                                )
                                .await
                                .expect("Failed to complete episode");

                            sleep(Duration::from_millis(5)).await;
                        }
                    }
                });

                handles.push(handle);
            }

            futures::future::join_all(handles).await;
        }
    }
}

fn benchmark_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");
    group.sample_size(5); // Fewer samples for memory benchmarks
    group.measurement_time(std::time::Duration::from_secs(60)); // Longer runs for memory analysis

    let scenarios = vec![
        MemoryScenario::SteadyState,
        MemoryScenario::BurstLoad,
        MemoryScenario::GradualGrowth,
        MemoryScenario::HighConcurrency,
    ];

    for scenario in scenarios {
        group.bench_with_input(
            BenchmarkId::new(scenario.name(), scenario.description()),
            &scenario,
            |b, scenario| {
                b.to_async(TokioExecutor).iter_custom(|iters| async move {
                    let mut total_time = std::time::Duration::ZERO;

                    for _ in 0..iters {
                        let (memory, _temp_dir) = setup_temp_memory().await;
                        let memory = Arc::new(memory);

                        let monitor = MemoryMonitor::new();

                        // Start monitoring
                        #[allow(clippy::excessive_nesting)]
                        let monitor_handle = tokio::spawn(async move {
                            let mut monitor = monitor;
                            let start = std::time::Instant::now();

                            while start.elapsed().as_secs() < 30 {
                                monitor.record_measurement();
                                sleep(Duration::from_millis(100)).await;
                            }

                            monitor
                        });

                        // Run the memory pressure scenario
                        let scenario_clone = scenario.clone();
                        #[allow(clippy::excessive_nesting)]
                        let scenario_handle = tokio::spawn(async move {
                            run_memory_pressure_scenario(memory, scenario_clone, 25).await;
                        });

                        let (monitor, _) = tokio::join!(monitor_handle, scenario_handle);
                        let monitor = monitor.unwrap();

                        // Record memory statistics
                        let peak_memory = monitor.get_peak_memory_mb();
                        let _avg_memory = monitor.get_average_memory_mb();
                        let _memory_variance = monitor.get_memory_variance();

                        // Use peak memory as the primary metric for comparison
                        total_time += std::time::Duration::from_secs_f32(peak_memory);
                    }

                    total_time
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_fragmentation");
    group.sample_size(10);

    // Test memory fragmentation by creating/deleting many small episodes
    group.bench_function("fragmentation_test", |b| {
        b.to_async(TokioExecutor).iter(|| async {
            let (memory, _temp_dir) = setup_temp_memory().await;
            let context = create_benchmark_context();

            // Create many small episodes then delete them (simulate fragmentation)
            let mut episode_ids = vec![];

            for i in 0..100 {
                let episode_id = memory
                    .start_episode(
                        format!("Fragmentation test episode {}", i),
                        context.clone(),
                        TaskType::CodeGeneration,
                    )
                    .await;

                let steps = generate_execution_steps(1);
                for step in steps {
                    memory.log_step(episode_id, step).await;
                }

                memory
                    .complete_episode(
                        episode_id,
                        TaskOutcome::Success {
                            verdict: format!("Fragmentation episode {}", i),
                            artifacts: vec![],
                        },
                    )
                    .await
                    .expect("Failed to complete episode");

                episode_ids.push(episode_id);

                // Delete every other episode to create fragmentation
                if i % 2 == 0 && !episode_ids.is_empty() {
                    // Note: This would require a delete method in the API
                    // For now, we just create fragmentation through uneven patterns
                }
            }

            // Final retrieval to test fragmentation impact
            let _results = memory
                .retrieve_relevant_context("Fragmentation test query".to_string(), context, 50)
                .await;
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_memory_pressure,
    benchmark_memory_fragmentation
);
criterion_main!(benches);
