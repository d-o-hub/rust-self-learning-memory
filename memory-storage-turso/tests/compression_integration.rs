//! Integration tests for compression functionality
//!
//! These tests validate:
//! - Compression is enabled by default
//! - Small payloads are not compressed
//! - Large payloads are compressed with 40% reduction
//! - Round-trip compression/decompression works
//! - Compression statistics are tracked

use memory_core::embeddings::EmbeddingStorageBackend;
use memory_storage_turso::TursoStorage;
use uuid::Uuid;

async fn setup_storage_with_embeddings() -> TursoStorage {
    // Use a shared cache file database
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().expect("Invalid path");

    let db = libsql::Builder::new_local(db_path_str)
        .build()
        .await
        .expect("Failed to create database");

    let storage = TursoStorage::from_database(db).expect("Failed to create storage");

    // Manually create embeddings table
    let conn = storage
        .get_connection()
        .await
        .expect("Failed to get connection");

    // Create table
    if let Err(e) = conn
        .execute(
            r#"
CREATE TABLE IF NOT EXISTS embeddings (
    embedding_id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    embedding_data TEXT NOT NULL,
    dimension INTEGER NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
"#,
            (),
        )
        .await
    {
        panic!("Failed to create embeddings table: {}", e);
    }

    // Create index
    if let Err(e) = conn
        .execute(
            r#"
CREATE INDEX IF NOT EXISTS idx_embeddings_item
ON embeddings(item_id, item_type)
"#,
            (),
        )
        .await
    {
        panic!("Failed to create embeddings index: {}", e);
    }

    // Verify table exists
    let check_result = conn
        .query("SELECT name FROM sqlite_master WHERE type='table' AND name='embeddings'", ())
        .await;
    match check_result {
        Ok(mut rows) => {
            if rows.next().await.transpose().is_none() {
                panic!("Table 'embeddings' was not created!");
            }
        }

    storage
}

#[tokio::test]
async fn test_compression_enabled_by_default() {
    // Test that compression is enabled by default
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let db_path = temp_file.path().to_str().expect("Invalid path");

    let db = libsql::Builder::new_local(db_path)
        .build()
        .await
        .expect("Failed to create database");

    let storage = TursoStorage::from_database(db).expect("Failed to create storage");

    // Verify compression stats are available (feature enabled)
    let stats = storage.compression_statistics();
    assert_eq!(stats.total_original_bytes, 0);
    assert_eq!(stats.total_compressed_bytes, 0);
}

#[tokio::test]
async fn test_small_embedding_not_compressed() {
    // Small embeddings should not be compressed (< 1KB threshold)
    let storage = setup_storage_with_embeddings().await;

    // Create a small embedding (384 floats = 1536 bytes, uncompressed)
    let small_embedding: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
    let episode_id = Uuid::new_v4();

    // Store the small embedding
    storage
        .store_episode_embedding(episode_id, small_embedding.clone())
        .await
        .expect("Failed to store small embedding");

    // Verify no compression occurred (should be skipped)
    let stats = storage.compression_statistics();
    assert_eq!(stats.compression_count, 0);
    assert_eq!(stats.skipped_count, 1);

    // Verify round-trip works
    let retrieved = storage
        .get_episode_embedding(episode_id)
        .await
        .expect("Failed to retrieve small embedding");

    assert_eq!(retrieved, Some(small_embedding));
}

#[tokio::test]
async fn test_large_embedding_compressed() {
    // Large embeddings should be compressed
    let storage = setup_storage_with_embeddings().await;

    // Create a large embedding (1536 floats = 6144 bytes)
    // Use sequential values that compress well
    let large_embedding: Vec<f32> = (0..1536).map(|i| i as f32 / 1536.0).collect();
    let episode_id = Uuid::new_v4();

    // Store the large embedding
    storage
        .store_episode_embedding(episode_id, large_embedding.clone())
        .await
        .expect("Failed to store large embedding");

    // Verify compression occurred
    let stats = storage.compression_statistics();
    assert_eq!(stats.compression_count, 1);
    assert_eq!(stats.skipped_count, 0);

    // Verify 40% bandwidth reduction target
    // Sequential floats compress very well (60-90%)
    let bandwidth_savings = stats.bandwidth_savings_percent();
    assert!(
        bandwidth_savings >= 40.0,
        "Expected >= 40% bandwidth reduction, got {}%",
        bandwidth_savings
    );

    println!("✅ Compression ratio: {:.2}", stats.compression_ratio());
    println!("✅ Bandwidth savings: {:.1}%", bandwidth_savings);

    // Verify round-trip works
    let retrieved = storage
        .get_episode_embedding(episode_id)
        .await
        .expect("Failed to retrieve large embedding");

    assert_eq!(retrieved, Some(large_embedding));
}

#[tokio::test]
async fn test_compression_ratio_calculation() {
    // Test that compression ratio is calculated correctly
    let storage = setup_storage_with_embeddings().await;

    // Create multiple large embeddings for better statistics
    for i in 0..5 {
        let embedding: Vec<f32> = (0..1536).map(|j| (i * 1536 + j) as f32).collect();
        let episode_id = Uuid::new_v4();

        storage
            .store_episode_embedding(episode_id, embedding)
            .await
            .expect("Failed to store embedding");
    }

    // Verify compression statistics
    let stats = storage.compression_statistics();
    assert_eq!(stats.compression_count, 5);

    // Compression ratio should be < 1.0 (compressed is smaller)
    assert!(stats.compression_ratio() < 1.0);
    assert!(stats.compression_ratio() > 0.0);

    // Bandwidth savings should be positive
    assert!(stats.bandwidth_savings_percent() > 0.0);

    println!("✅ Compression ratio: {:.2}", stats.compression_ratio());
    println!(
        "✅ Bandwidth savings: {:.1}%",
        stats.bandwidth_savings_percent()
    );
}

#[tokio::test]
async fn test_reset_compression_statistics() {
    // Test that statistics can be reset
    let storage = setup_storage_with_embeddings().await;

    // Create and store an embedding
    let embedding: Vec<f32> = (0..1536).map(|i| i as f32).collect();
    let episode_id = Uuid::new_v4();

    storage
        .store_episode_embedding(episode_id, embedding)
        .await
        .expect("Failed to store embedding");

    // Verify statistics are recorded
    let stats = storage.compression_statistics();
    assert!(stats.compression_count > 0);

    // Reset statistics
    storage.reset_compression_statistics();

    // Verify statistics are cleared
    let stats = storage.compression_statistics();
    assert_eq!(stats.compression_count, 0);
    assert_eq!(stats.total_original_bytes, 0);
    assert_eq!(stats.total_compressed_bytes, 0);
}
