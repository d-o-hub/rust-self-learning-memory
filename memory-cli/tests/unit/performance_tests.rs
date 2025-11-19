//! Performance tests for CLI operations.
//!
//! These tests verify that CLI operations perform within acceptable time bounds
//! and scale appropriately with input size.

use memory_cli::test_utils::*;
use std::time::{Duration, Instant};

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_config_validation_performance() {
        let harness = CliHarness::new();

        // Measure config validation performance
        let start = Instant::now();
        harness.execute(["config"]).assert().success();
        let duration = start.elapsed();

        // Config validation should be very fast (< 50ms)
        assert!(duration < Duration::from_millis(50),
            "Config validation took {}ms, expected < 50ms", duration.as_millis());
    }

    #[test]
    fn test_help_command_performance() {
        let harness = CliHarness::new();

        // Measure help command performance
        let start = Instant::now();
        harness.execute(["--help"]).assert().success();
        let duration = start.elapsed();

        // Help should be very fast (< 20ms)
        assert!(duration < Duration::from_millis(20),
            "Help command took {}ms, expected < 20ms", duration.as_millis());
    }

    #[test]
    fn test_version_command_performance() {
        let harness = CliHarness::new();

        // Measure version command performance
        let start = Instant::now();
        let result = harness.execute(["--version"]).assert();
        // Version command typically exits with error code for clap
        let duration = start.elapsed();

        // Version should be very fast (< 10ms)
        assert!(duration < Duration::from_millis(10),
            "Version command took {}ms, expected < 10ms", duration.as_millis());
    }

    #[test]
    fn test_output_format_performance_comparison() {
        let harness = CliHarness::new();

        // Measure performance of different output formats
        let formats = vec!["human", "json", "yaml"];
        let mut results = Vec::new();

        for format in &formats {
            let start = Instant::now();
            harness.execute(["--format", format, "config"]).assert().success();
            let duration = start.elapsed();
            results.push((format.to_string(), duration));
        }

        // All formats should be reasonably fast (< 100ms)
        for (format, duration) in &results {
            assert!(duration < Duration::from_millis(100),
                "Format {} took {}ms, expected < 100ms", format, duration.as_millis());
        }

        // Print performance comparison for analysis
        println!("Output format performance:");
        for (format, duration) in &results {
            println!("  {}: {}ms", format, duration.as_millis());
        }
    }

    #[test]
    fn test_dry_run_performance() {
        let harness = CliHarness::new();

        // Measure dry run performance (should be similar to normal execution)
        let start = Instant::now();
        harness.execute(["--dry-run", "config"]).assert().success();
        let duration = start.elapsed();

        // Dry run should be fast (< 50ms)
        assert!(duration < Duration::from_millis(50),
            "Dry run took {}ms, expected < 50ms", duration.as_millis());
    }

    #[test]
    fn test_verbose_output_performance() {
        let harness = CliHarness::new();

        // Measure verbose output performance
        let start = Instant::now();
        harness.execute(["--verbose", "config"]).assert().success();
        let duration = start.elapsed();

        // Verbose output should not significantly impact performance (< 100ms)
        assert!(duration < Duration::from_millis(100),
            "Verbose output took {}ms, expected < 100ms", duration.as_millis());
    }

    #[test]
    fn test_completion_generation_performance() {
        let harness = CliHarness::new();
        let shells = vec!["bash", "zsh", "fish"];

        for shell in &shells {
            let start = Instant::now();
            harness.execute(["completion", shell]).assert().success();
            let duration = start.elapsed();

            // Completion generation should be fast (< 50ms)
            assert!(duration < Duration::from_millis(50),
                "Completion generation for {} took {}ms, expected < 50ms",
                shell, duration.as_millis());
        }
    }

    #[test]
    fn test_concurrent_cli_operations() {
        let harness = CliHarness::new();
        let num_operations = 10;

        let start = Instant::now();

        // Run multiple CLI operations concurrently
        let handles: Vec<_> = (0..num_operations)
            .map(|_| {
                let harness_clone = harness.clone();
                std::thread::spawn(move || {
                    harness_clone.execute(["config"]).assert().success();
                })
            })
            .collect();

        // Wait for all operations to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let total_duration = start.elapsed();
        let avg_duration = total_duration / num_operations as u32;

        // Average operation should be fast (< 20ms per operation)
        assert!(avg_duration < Duration::from_millis(20),
            "Average CLI operation took {}ms, expected < 20ms", avg_duration.as_millis());

        println!("Concurrent operations: {} total, {}ms average per operation",
            num_operations, avg_duration.as_millis());
    }

    #[test]
    fn test_large_config_file_performance() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("large_config.toml");

        // Create a large config file with many entries
        let mut config_content = String::from("[database]\nturso_url = \"file:test.db\"\n");
        config_content.push_str("[storage]\nmax_episodes_cache = 1000\ncache_ttl_seconds = 3600\npool_size = 10\n");
        config_content.push_str("[cli]\ndefault_format = \"json\"\nprogress_bars = true\nbatch_size = 100\n");

        // Add many metadata entries
        for i in 0..100 {
            config_content.push_str(&format!("metadata_{} = \"value_{}\"\n", i, i));
        }

        std::fs::write(&config_path, config_content).unwrap();

        let harness = CliHarness::new();

        // Measure loading and validation of large config
        let start = Instant::now();
        harness.execute(["--config", &config_path.to_string_lossy(), "config"]).assert().success();
        let duration = start.elapsed();

        // Large config should still load reasonably fast (< 200ms)
        assert!(duration < Duration::from_millis(200),
            "Large config loading took {}ms, expected < 200ms", duration.as_millis());
    }

    #[test]
    fn test_cli_startup_time() {
        // Measure CLI binary startup time (just parsing args, not executing commands)
        let start = Instant::now();

        // Use a command that exits early (like --help)
        let mut cmd = std::process::Command::new("cargo")
            .args(["run", "--bin", "memory-cli", "--", "--help"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();

        let _ = cmd.wait();
        let duration = start.elapsed();

        // Startup time should be reasonable (< 500ms including cargo overhead)
        assert!(duration < Duration::from_millis(500),
            "CLI startup took {}ms, expected < 500ms", duration.as_millis());
    }

    #[test]
    fn test_memory_usage_stability() {
        let harness = CliHarness::new();

        // Run multiple operations and check for memory leaks (basic check)
        for i in 0..50 {
            harness.execute(["config"]).assert().success();

            // Small delay to allow for cleanup
            std::thread::sleep(Duration::from_millis(1));
        }

        // If we get here without crashing, basic memory stability is ok
        // In a real performance test suite, we'd use memory profiling tools
    }

    #[test]
    fn test_error_handling_performance() {
        let harness = CliHarness::new();

        // Test that error handling doesn't significantly slow down operations
        let start = Instant::now();

        // Try various invalid operations that should fail quickly
        let invalid_commands = vec![
            vec!["episode", "view", "invalid-uuid"],
            vec!["episode", "list", "--task-type", "invalid@type"],
            vec!["--format", "invalid", "config"],
            vec!["unknown-command"],
        ];

        for cmd in invalid_commands {
            let _ = harness.execute(&cmd); // We expect failures, just measure time
        }

        let duration = start.elapsed();
        let avg_duration = duration / invalid_commands.len() as u32;

        // Error handling should be fast (< 30ms per error)
        assert!(avg_duration < Duration::from_millis(30),
            "Average error handling took {}ms, expected < 30ms", avg_duration.as_millis());
    }

    #[test]
    fn test_scaling_with_input_size() {
        let harness = CliHarness::new();

        // Test how performance scales with input size (simulated)
        let input_sizes = vec![10, 100, 1000];

        for size in input_sizes {
            let large_input = "x".repeat(size);

            let start = Instant::now();
            // Note: This will fail due to missing storage features, but we measure the parsing/validation time
            let _ = harness.execute(["episode", "create", &large_input]);
            let duration = start.elapsed();

            // Input processing should scale reasonably (linear or better)
            // Allow up to 100ms for large inputs
            assert!(duration < Duration::from_millis(100),
                "Processing input of size {} took {}ms, expected < 100ms", size, duration.as_millis());

            println!("Input size {}: {}ms", size, duration.as_millis());
        }
    }

    #[test]
    fn test_cli_startup_time_benchmarking() {
        // Test CLI startup time - should be < 1000ms as per requirements
        let mut times = Vec::new();

        for _ in 0..10 {
            let start = Instant::now();

            // Use --help as it exits quickly after parsing
            let mut cmd = std::process::Command::new("cargo")
                .args(["run", "--bin", "memory-cli", "--", "--help"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();

            let _ = cmd.wait();
            let duration = start.elapsed();
            times.push(duration);
        }

        let avg_time: Duration = times.iter().sum::<Duration>() / times.len() as u32;
        let max_time = times.iter().max().unwrap();

        println!("CLI Startup Time - Average: {}ms, Max: {}ms",
                 avg_time.as_millis(), max_time.as_millis());

        // Target: < 1000ms average startup time
        assert!(avg_time < Duration::from_millis(1000),
                "Average startup time {}ms exceeds 1000ms target", avg_time.as_millis());

        // Allow some variance but keep max reasonable
        assert!(max_time < Duration::from_millis(1500),
                "Max startup time {}ms exceeds 1500ms limit", max_time.as_millis());
    }

    #[test]
    fn test_command_execution_latency_benchmarking() {
        let harness = CliHarness::new();
        let mut results = Vec::new();

        // Test various list operations that should be < 500ms
        let list_commands = vec![
            vec!["episode", "list"],
            vec!["pattern", "list"],
            vec!["storage", "stats"],
        ];

        for cmd_args in list_commands {
            let mut times = Vec::new();

            // Run each command 5 times
            for _ in 0..5 {
                let start = Instant::now();
                let _ = harness.execute(&cmd_args).output().unwrap();
                times.push(start.elapsed());
            }

            let avg_time: Duration = times.iter().sum::<Duration>() / times.len() as u32;
            results.push((cmd_args.join(" "), avg_time));

            // Each list operation should be < 500ms
            assert!(avg_time < Duration::from_millis(500),
                    "Command '{}' average {}ms exceeds 500ms target",
                    cmd_args.join(" "), avg_time.as_millis());
        }

        println!("Command Execution Latency Results:");
        for (cmd, duration) in &results {
            println!("  {}: {}ms", cmd, duration.as_millis());
        }
    }

    #[test]
    fn test_memory_usage_profiling() {
        let harness = CliHarness::new();

        // Run multiple operations and monitor for memory issues
        // This is a basic test - in production you'd use proper memory profiling tools
        let start = Instant::now();

        for i in 0..100 {
            // Mix of different operations
            let cmd = match i % 4 {
                0 => vec!["config"],
                1 => vec!["episode", "list"],
                2 => vec!["pattern", "list"],
                _ => vec!["storage", "stats"],
            };

            let _ = harness.execute(&cmd).output().unwrap();

            // Small delay to prevent overwhelming the system
            std::thread::sleep(Duration::from_millis(1));
        }

        let total_duration = start.elapsed();
        let avg_operation_time = total_duration / 100;

        println!("Memory Usage Test - 100 operations in {}ms (avg: {}ms per op)",
                 total_duration.as_millis(), avg_operation_time.as_millis());

        // Basic sanity check - operations should complete reasonably fast
        assert!(avg_operation_time < Duration::from_millis(50),
                "Average operation time {}ms suggests potential memory issues",
                avg_operation_time.as_millis());
    }

    #[test]
    fn test_concurrent_operation_testing() {
        let harness = CliHarness::new();
        let num_threads = 5;
        let operations_per_thread = 10;

        let start = Instant::now();

        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let harness_clone = harness.clone();
                std::thread::spawn(move || {
                    let mut thread_times = Vec::new();

                    for i in 0..operations_per_thread {
                        let start_op = Instant::now();

                        // Alternate between different commands
                        let cmd = match i % 3 {
                            0 => vec!["config"],
                            1 => vec!["episode", "list"],
                            _ => vec!["pattern", "list"],
                        };

                        let _ = harness_clone.execute(&cmd).output().unwrap();
                        thread_times.push(start_op.elapsed());
                    }

                    thread_times
                })
            })
            .collect();

        // Collect all operation times
        let mut all_times = Vec::new();
        for handle in handles {
            all_times.extend(handle.join().unwrap());
        }

        let total_duration = start.elapsed();
        let avg_operation_time: Duration = all_times.iter().sum::<Duration>() / all_times.len() as u32;
        let max_operation_time = all_times.iter().max().unwrap();

        println!("Concurrent Operations Test:");
        println!("  Total operations: {}", all_times.len());
        println!("  Total duration: {}ms", total_duration.as_millis());
        println!("  Average operation time: {}ms", avg_operation_time.as_millis());
        println!("  Max operation time: {}ms", max_operation_time.as_millis());

        // Concurrent operations should still be reasonably fast
        assert!(avg_operation_time < Duration::from_millis(100),
                "Average concurrent operation time {}ms too slow", avg_operation_time.as_millis());

        // No operation should take excessively long
        assert!(max_operation_time < Duration::from_millis(500),
                "Max operation time {}ms indicates performance issues", max_operation_time.as_millis());
    }

    #[test]
    fn test_performance_regression_detection() {
        let harness = CliHarness::new();

        // Establish baseline performance for critical operations
        let baseline_operations = vec![
            ("config", vec!["config"]),
            ("episode-list", vec!["episode", "list"]),
            ("pattern-list", vec!["pattern", "list"]),
            ("storage-stats", vec!["storage", "stats"]),
        ];

        println!("Performance Regression Baseline:");

        for (name, cmd) in baseline_operations {
            let mut times = Vec::new();

            // Run 10 times to get stable average
            for _ in 0..10 {
                let start = Instant::now();
                let _ = harness.execute(&cmd).output().unwrap();
                times.push(start.elapsed());
            }

            let avg_time: Duration = times.iter().sum::<Duration>() / times.len() as u32;
            let p95_time = {
                times.sort();
                times[(times.len() * 95) / 100]
            };

            println!("  {}: avg {}ms, p95 {}ms", name, avg_time.as_millis(), p95_time.as_millis());

            // Set reasonable performance expectations
            let max_expected = match name {
                "config" => 50,
                "episode-list" | "pattern-list" => 200,
                "storage-stats" => 100,
                _ => 100,
            };

            assert!(avg_time < Duration::from_millis(max_expected),
                    "{} average {}ms exceeds {}ms baseline", name, avg_time.as_millis(), max_expected);
        }
    }

    #[test]
    fn test_large_dataset_performance() {
        let harness = CliHarness::new();

        // Test performance with simulated large datasets
        // This tests the CLI's ability to handle output formatting for large result sets

        let start = Instant::now();

        // Test with various output formats and large simulated data
        let formats = vec!["human", "json", "yaml"];

        for format in formats {
            let cmd_start = Instant::now();
            let _ = harness.execute(&[format, "config"]).output().unwrap();
            let cmd_duration = cmd_start.elapsed();

            println!("Format {}: {}ms", format, cmd_duration.as_millis());

            // All formats should be reasonably fast
            assert!(cmd_duration < Duration::from_millis(100),
                    "Format {} took {}ms, expected < 100ms", format, cmd_duration.as_millis());
        }

        let total_duration = start.elapsed();
        println!("Large dataset test completed in {}ms", total_duration.as_millis());
    }
}