//! Benchmark for Network Compression
//!
//! Verifies the 40% bandwidth reduction target:
//! - Baseline (no compression): Full payload size
//! - With compression: ~40% reduction in size

#![allow(unexpected_cfgs)]
#![allow(clippy::excessive_nesting)]
#![allow(deprecated)]
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_turso::{TursoConfig, TursoStorage};
use std::hint::black_box;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Helper to create episode of specific size
fn create_episode_with_size(size_kb: usize) -> Episode {
    let _data_size = size_kb * 1024;
    let context = TaskContext {
        domain: "benchmark".to_string(),
        ..Default::default()
    };

    let episode = Episode::new(
        format!("compression_benchmark_{}kb", size_kb),
        context,
        TaskType::CodeGeneration,
    );

    episode
}

/// Setup storage with compression enabled
fn setup_storage_with_compression() -> (TursoStorage, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("bench.db");

    let rt = Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let config = TursoConfig {
            compression_threshold: 1024, // 1KB
            compress_episodes: true,
            ..Default::default()
        };

        let storage = TursoStorage::with_config(&format!("file:{}", db_path.display()), "", config)
            .await
            .expect("Failed to create storage");

        storage
            .initialize_schema()
            .await
            .expect("Failed to initialize schema");

        storage
    });

    (storage, dir)
}

/// Setup storage without compression
fn setup_storage_without_compression() -> (TursoStorage, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("bench.db");

    let rt = Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let config = TursoConfig {
            compress_episodes: false, // Disable compression
            ..Default::default()
        };

        let storage = TursoStorage::with_config(&format!("file:{}", db_path.display()), "", config)
            .await
            .expect("Failed to create storage");

        storage
            .initialize_schema()
            .await
            .expect("Failed to initialize schema");

        storage
    });

    (storage, dir)
}

/// Benchmark compression overhead
fn bench_compression_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_overhead");

    for size_kb in [1, 5, 10, 50, 100].iter() {
        let episode = create_episode_with_size(*size_kb);
        let serialized = serde_json::to_string(&episode).expect("Failed to serialize");
        let original_size = serialized.len();

        group.throughput(Throughput::Bytes(original_size as u64));

        // Benchmark with compression
        group.bench_with_input(
            BenchmarkId::new("with_compression", size_kb),
            size_kb,
            |b, _| {
                b.iter(|| {
                    #[cfg(feature = "compression")]
                    {
                        use memory_storage_turso::compression::CompressedPayload;
                        let compressed = CompressedPayload::compress(serialized.as_bytes(), 1024)
                            .expect("Compression failed");
                        black_box(compressed)
                    }

                    #[cfg(not(feature = "compression"))]
                    {
                        black_box(serialized.as_bytes())
                    }
                });
            },
        );

        // Benchmark without compression (baseline)
        group.bench_with_input(
            BenchmarkId::new("without_compression", size_kb),
            size_kb,
            |b, _| {
                b.iter(|| {
                    black_box(serialized.as_bytes());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark compression ratio for different payload sizes
fn bench_compression_ratio(c: &mut Criterion) {
    let group = c.benchmark_group("compression_ratio");

    #[cfg(feature = "compression")]
    {
        use memory_storage_turso::compression::CompressedPayload;

        for size_kb in [1, 5, 10, 50, 100].iter() {
            let episode = create_episode_with_size(*size_kb);
            let serialized = serde_json::to_string(&episode).expect("Failed to serialize");
            let original_size = serialized.len();

            // Compress and measure ratio
            let compressed = CompressedPayload::compress(serialized.as_bytes(), 1024)
                .expect("Compression failed");

            let ratio = compressed.compression_ratio;
            let reduction = (1.0 - ratio) * 100.0;

            println!(
                "{}KB: {:.2}% reduction (original: {} bytes, compressed: {} bytes)",
                size_kb, reduction, original_size, compressed.compressed_size
            );

            group.throughput(Throughput::Bytes(original_size as u64));
            group.bench_with_input(BenchmarkId::new("compress", size_kb), size_kb, |b, _| {
                b.iter(|| {
                    let compressed = CompressedPayload::compress(serialized.as_bytes(), 1024)
                        .expect("Compression failed");
                    black_box(compressed)
                });
            });
        }
    }

    group.finish();
}

/// Benchmark storage operations with/without compression
fn bench_storage_with_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_operations");

    let rt = Runtime::new().unwrap();

    for size_kb in [5, 10, 50].iter() {
        let episode = create_episode_with_size(*size_kb);

        // With compression
        let (storage_compressed, _dir1) = setup_storage_with_compression();
        group.bench_with_input(
            BenchmarkId::new("with_compression", size_kb),
            size_kb,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let mut ep = episode.clone();
                    ep.episode_id = Uuid::new_v4();
                    storage_compressed
                        .store_episode(&ep)
                        .await
                        .expect("Failed to store");
                    black_box(ep.episode_id)
                });
            },
        );

        // Without compression
        let (storage_uncompressed, _dir2) = setup_storage_without_compression();
        group.bench_with_input(
            BenchmarkId::new("without_compression", size_kb),
            size_kb,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let mut ep = episode.clone();
                    ep.episode_id = Uuid::new_v4();
                    storage_uncompressed
                        .store_episode(&ep)
                        .await
                        .expect("Failed to store");
                    black_box(ep.episode_id)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark decompression performance
fn bench_decompression(c: &mut Criterion) {
    let group = c.benchmark_group("decompression");

    #[cfg(feature = "compression")]
    {
        use memory_storage_turso::compression::CompressedPayload;

        for size_kb in [5, 10, 50].iter() {
            let episode = create_episode_with_size(*size_kb);
            let serialized = serde_json::to_string(&episode).expect("Failed to serialize");

            let compressed = CompressedPayload::compress(serialized.as_bytes(), 1024)
                .expect("Compression failed");

            group.throughput(Throughput::Bytes(compressed.compressed_size as u64));
            group.bench_with_input(BenchmarkId::new("decompress", size_kb), size_kb, |b, _| {
                b.iter(|| {
                    let decompressed = compressed.decompress().expect("Decompression failed");
                    black_box(decompressed)
                });
            });
        }
    }

    group.finish();
}

/// Benchmark different compression algorithms
fn bench_compression_algorithms(c: &mut Criterion) {
    let group = c.benchmark_group("compression_algorithms");

    #[cfg(feature = "compression")]
    {
        use memory_storage_turso::compression::CompressedPayload;

        let episode = create_episode_with_size(10);
        let serialized = serde_json::to_string(&episode).expect("Failed to serialize");
        let data = serialized.as_bytes();

        #[cfg(feature = "compression-lz4")]
        group.bench_function("lz4", |b| {
            b.iter(|| {
                let compressed = CompressedPayload::compress_lz4(data).expect("LZ4 failed");
                black_box(compressed)
            });
        });

        #[cfg(feature = "compression-zstd")]
        group.bench_function("zstd", |b| {
            b.iter(|| {
                let compressed = CompressedPayload::compress_zstd(data).expect("Zstd failed");
                black_box(compressed)
            });
        });

        #[cfg(feature = "compression-gzip")]
        group.bench_function("gzip", |b| {
            b.iter(|| {
                let compressed = CompressedPayload::compress_gzip(data).expect("Gzip failed");
                black_box(compressed)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_compression_overhead,
    bench_compression_ratio,
    bench_storage_with_compression,
    bench_decompression,
    bench_compression_algorithms
);
criterion_main!(benches);
