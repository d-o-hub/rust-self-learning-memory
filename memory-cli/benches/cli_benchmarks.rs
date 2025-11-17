//! Performance benchmarks for CLI operations
//!
//! These benchmarks measure the performance of critical CLI paths
//! to ensure they meet performance requirements.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memory_cli::test_utils::*;
use std::time::Duration;

fn cli_startup_benchmark(c: &mut Criterion) {
    let harness = CliHarness::new();

    c.bench_function("cli_help_command", |b| {
        b.iter(|| {
            harness.execute(black_box(["--help"])).assert().success();
        });
    });
}

fn cli_config_benchmark(c: &mut Criterion) {
    let harness = CliHarness::new();

    c.bench_function("cli_config_validation", |b| {
        b.iter(|| {
            harness.execute(black_box(["config"])).assert().success();
        });
    });
}

fn cli_output_format_benchmark(c: &mut Criterion) {
    let harness = CliHarness::new();

    let formats = ["human", "json", "yaml"];

    for format in formats {
        c.bench_function(format!("cli_config_{}_output", format), |b| {
            b.iter(|| {
                harness
                    .execute(black_box(["--format", format, "config"]))
                    .assert()
                    .success();
            });
        });
    }
}

fn cli_completion_benchmark(c: &mut Criterion) {
    let harness = CliHarness::new();

    let shells = ["bash", "zsh", "fish"];

    for shell in shells {
        c.bench_function(format!("cli_completion_{}", shell), |b| {
            b.iter(|| {
                harness
                    .execute(black_box(["completion", shell]))
                    .assert()
                    .success();
            });
        });
    }
}

fn cli_error_handling_benchmark(c: &mut Criterion) {
    let harness = CliHarness::new();

    c.bench_function("cli_invalid_command", |b| {
        b.iter(|| {
            harness
                .execute(black_box(["invalid-command"]))
                .assert()
                .failure();
        });
    });
}

fn cli_dry_run_benchmark(c: &mut Criterion) {
    let harness = CliHarness::new();

    c.bench_function("cli_dry_run_episode_create", |b| {
        b.iter(|| {
            harness
                .execute(black_box(["--dry-run", "episode", "create", "benchmark task"]))
                .assert()
                .success();
        });
    });
}

criterion_group! {
    name = cli_benchmarks;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1));
    targets =
        cli_startup_benchmark,
        cli_config_benchmark,
        cli_output_format_benchmark,
        cli_completion_benchmark,
        cli_error_handling_benchmark,
        cli_dry_run_benchmark
}

criterion_main!(cli_benchmarks);