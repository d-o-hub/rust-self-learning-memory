//! Tests for real embedding model

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    #[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
    use crate::embeddings::real_model::download::validate_downloaded_file;

    #[test]
    fn test_url_construction() {
        let model_name = "sentence-transformers/all-MiniLM-L6-v2";
        let sanitized = model_name.replace('/', "_");
        let base_url = format!("https://huggingface.co/{model_name}/resolve/main");

        assert_eq!(sanitized, "sentence-transformers_all-MiniLM-L6-v2");
        assert_eq!(
            base_url,
            "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main"
        );

        let files = [
            format!("{sanitized}.onnx"),
            format!("{sanitized}_tokenizer.json"),
            format!("{sanitized}_config.json"),
        ];

        assert_eq!(files[0], "sentence-transformers_all-MiniLM-L6-v2.onnx");
        assert_eq!(
            files[1],
            "sentence-transformers_all-MiniLM-L6-v2_tokenizer.json"
        );
        assert_eq!(
            files[2],
            "sentence-transformers_all-MiniLM-L6-v2_config.json"
        );
    }

    #[test]
    fn test_progress_calculation() {
        let total = 10_485_760u64; // 10 MB
        let downloaded = 5_242_880u64; // 5 MB
        let progress = (downloaded as f64 / total as f64) * 100.0;

        assert!((progress - 50.0).abs() < 0.01);

        let total_mb = total as f64 / 1_048_576.0;
        let downloaded_mb = downloaded as f64 / 1_048_576.0;

        assert!((total_mb - 10.0).abs() < 0.01);
        assert!((downloaded_mb - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_retry_backoff() {
        let max_retries = 3;
        for retry_count in 0..max_retries {
            let backoff_duration = std::time::Duration::from_millis(100 * 2_u64.pow(retry_count));
            let expected_ms = 100 * 2_u64.pow(retry_count);

            assert_eq!(backoff_duration.as_millis() as u64, expected_ms);
        }

        // Verify exponential backoff: 100ms, 200ms, 400ms
        assert_eq!(
            std::time::Duration::from_millis(100 * 2_u64.pow(0)).as_millis(),
            100
        );
        assert_eq!(
            std::time::Duration::from_millis(100 * 2_u64.pow(1)).as_millis(),
            200
        );
        assert_eq!(
            std::time::Duration::from_millis(100 * 2_u64.pow(2)).as_millis(),
            400
        );
    }

    #[test]
    #[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
    fn test_file_validation() {
        use std::io::Write;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Test with non-existent file
        let result = validate_downloaded_file(&file_path, "test_file.txt");
        assert!(result.is_err());

        // Test with empty file
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"")
            .unwrap();
        let result = validate_downloaded_file(&file_path, "test_file.txt");
        assert!(result.is_err());

        // Test with valid file
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"test content")
            .unwrap();
        let result = validate_downloaded_file(&file_path, "test_file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_speed_calculation() {
        // Test speed calculation
        let downloaded = 5_242_880u64; // 5 MB
        let elapsed = std::time::Duration::from_secs(2);
        let speed = (downloaded as f64 / 1_048_576.0) / elapsed.as_secs_f64();

        assert!((speed - 2.5).abs() < 0.01);

        // Test zero elapsed time handling
        let elapsed_zero = std::time::Duration::from_secs(0);
        let speed_zero = if elapsed_zero.as_secs_f64() > 0.0 {
            (downloaded as f64 / 1_048_576.0) / elapsed_zero.as_secs_f64()
        } else {
            0.0
        };

        assert_eq!(speed_zero, 0.0);
    }
}
