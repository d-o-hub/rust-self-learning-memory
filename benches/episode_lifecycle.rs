//! Benchmarks for episode lifecycle operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memory_core::*;
use test_utils::*;

fn benchmark_episode_creation(c: &mut Criterion) {
    c.bench_function("episode_creation", |b| {
        b.iter(|| {
            create_test_episode(black_box("Benchmark task"))
        });
    });
}

fn benchmark_add_step(c: &mut Criterion) {
    c.bench_function("add_execution_step", |b| {
        let mut episode = create_test_episode("Test");
        let mut counter = 0;
        
        b.iter(|| {
            counter += 1;
            let step = create_test_step(counter);
            episode.add_step(black_box(step));
        });
    });
}

fn benchmark_episode_completion(c: &mut Criterion) {
    c.bench_function("episode_completion", |b| {
        b.iter(|| {
            let mut episode = create_test_episode("Test");
            for i in 0..5 {
                episode.add_step(create_test_step(i + 1));
            }
            let outcome = TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            };
            episode.complete(black_box(outcome));
        });
    });
}

fn benchmark_reward_calculation(c: &mut Criterion) {
    let episode = create_completed_episode("Test", true);
    
    c.bench_function("reward_calculation", |b| {
        b.iter(|| {
            compute_reward_score(black_box(&episode))
        });
    });
}

fn benchmark_reflection_generation(c: &mut Criterion) {
    let episode = create_completed_episode("Test", true);
    
    c.bench_function("reflection_generation", |b| {
        b.iter(|| {
            generate_reflection(black_box(&episode))
        });
    });
}

criterion_group!(
    benches,
    benchmark_episode_creation,
    benchmark_add_step,
    benchmark_episode_completion,
    benchmark_reward_calculation,
    benchmark_reflection_generation
);
criterion_main!(benches);
