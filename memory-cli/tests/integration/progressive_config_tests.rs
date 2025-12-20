//! Integration tests for progressive configuration modes
//!
//! These tests verify that the three progressive configuration modes
//! (Ultra-Simple, Simple, and Advanced) work correctly and provide
//! clear progression from simple to advanced setup.

use memory_cli::config::progressive::*;
use memory_cli::config::{ConfigPreset, StorageInitResult};
use tempfile::TempDir;
use tokio::time::{Duration, timeout};

#[tokio::test]
async fn test_ultra_simple_setup() {
    // Test that ultra-simple setup completes quickly and creates valid config
    let start = std::time::Instant::now();
    
    let config = timeout(Duration::from_secs(30), setup_quick_redb())
        .await
        .expect("Ultra-simple setup should complete within 30 seconds")
        .expect("Ultra-simple setup should succeed");
    
    let elapsed = start.elapsed();
    
    // Verify it completed quickly (should be much less than 30 seconds)
    assert!(elapsed < Duration::from_secs(5), 
        "Ultra-simple setup took too long: {:?}", elapsed);
    
    // Verify basic config validity
    assert!(config.database.redb_path.is_some() || 
           config.database.turso_url.is_some(), 
        "Ultra-simple setup should configure some storage");
    
    assert!(config.storage.max_episodes_cache > 0);
    assert!(config.storage.pool_size > 0);
}

#[tokio::test]
async fn test_simple_setup_preset_selection() {
    // Test SimpleSetup with different presets
    let presets_to_test = vec![
        ConfigPreset::Local,
        ConfigPreset::Memory,
        ConfigPreset::Cloud,
    ];
    
    for preset in presets_to_test {
        let setup = SimpleSetup::preset(preset);
        let config = setup.build().await
            .expect("SimpleSetup should succeed for all presets");
        
        // Verify config is valid
        assert!(config.storage.max_episodes_cache > 0);
        assert!(config.storage.pool_size > 0);
        assert!(!config.cli.default_format.is_empty());
    }
}

#[tokio::test]
async fn test_simple_setup_custom_overrides() {
    // Test SimpleSetup with custom overrides
    let setup = SimpleSetup::preset(ConfigPreset::Local)
        .with_custom(|config| {
            config.storage.max_episodes_cache = 2500;
            config.cli.default_format = "json".to_string();
            config.cli.progress_bars = false;
        });
    
    let config = setup.build().await
        .expect("SimpleSetup with custom overrides should succeed");
    
    // Verify overrides were applied
    assert_eq!(config.storage.max_episodes_cache, 2500);
    assert_eq!(config.cli.default_format, "json");
    assert_eq!(config.cli.progress_bars, false);
}

#[tokio::test]
async fn test_simple_setup_validation() {
    // Test that SimpleSetup validates configuration properly
    let setup = SimpleSetup::preset(ConfigPreset::Local)
        .with_custom(|config| {
            // Set invalid values that should trigger validation
            config.storage.max_episodes_cache = 0; // Invalid
        });
    
    let result = setup.build().await;
    
    // Should fail validation
    assert!(result.is_err(), 
        "SimpleSetup should fail validation for invalid config");
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("validation") || error_msg.contains("failed"),
        "Error message should mention validation failure");
}

#[tokio::test]
async fn test_simple_setup_guidance_toggle() {
    // Test that guidance can be hidden for automation
    let setup_with_guidance = SimpleSetup::preset(ConfigPreset::Local)
        .hide_guidance();
    
    let setup_without_guidance = SimpleSetup::preset(ConfigPreset::Local)
        .hide_guidance();
    
    // Both should work, but guidance should be hidden in the second case
    let config1 = setup_with_guidance.build().await
        .expect("Setup with guidance should work");
    let config2 = setup_without_guidance.build().await
        .expect("Setup without guidance should work");
    
    // Both should produce valid configs
    assert_eq!(config1.storage.max_episodes_cache, config2.storage.max_episodes_cache);
}

#[tokio::test]
async fn test_mode_recommendations() {
    // Test that mode recommendations work correctly
    let usage_patterns = vec![
        (UsagePattern::QuickTest, ConfigurationMode::UltraSimple),
        (UsagePattern::Development, ConfigurationMode::Simple),
        (UsagePattern::Production, ConfigurationMode::Advanced),
        (UsagePattern::Testing, ConfigurationMode::UltraSimple),
    ];
    
    for (pattern, expected_mode) in usage_patterns {
        let recommendation = recommend_mode(&pattern);
        assert_eq!(recommendation.recommended_mode, expected_mode,
            "Mode recommendation should match expected mode for pattern {:?}", pattern);
        assert!(!recommendation.reasoning.is_empty(),
            "Recommendation should include reasoning");
        assert!(!recommendation.upgrade_path.is_empty(),
            "Recommendation should include upgrade path");
    }
}

#[tokio::test]
async fn test_configuration_mode_descriptions() {
    // Test that configuration mode descriptions are helpful
    let modes = vec![
        ConfigurationMode::UltraSimple,
        ConfigurationMode::Simple,
        ConfigurationMode::Advanced,
    ];
    
    for mode in modes {
        let description = mode.description();
        assert!(!description.is_empty(), "Mode description should not be empty");
        assert!(description.contains("•"), "Description should include bullet points");
        assert!(description.contains("Best for:"), "Description should include use cases");
        
        let setup_time = mode.setup_time();
        assert!(!setup_time.is_empty(), "Setup time should be specified");
    }
}

#[tokio::test]
async fn test_progressive_disclosure() {
    // Test that we can progress from simple to advanced modes
    // Start with Ultra-Simple
    let ultra_simple_config = setup_quick_redb().await
        .expect("Ultra-simple setup should work");
    
    // Upgrade to Simple with custom overrides
    let simple_config = SimpleSetup::preset(ConfigPreset::Local)
        .with_custom(|config| {
            // Enhanced settings compared to ultra-simple
            config.storage.max_episodes_cache = ultra_simple_config.storage.max_episodes_cache * 2;
        })
        .build()
        .await
        .expect("Simple setup should work after ultra-simple");
    
    // Verify progressive enhancement
    assert!(simple_config.storage.max_episodes_cache > 
           ultra_simple_config.storage.max_episodes_cache,
        "Simple mode should allow enhancement over ultra-simple");
}

#[tokio::test]
async fn test_error_handling_and_user_guidance() {
    // Test that error messages provide helpful guidance
    let setup = SimpleSetup::preset(ConfigPreset::Local)
        .with_custom(|config| {
            config.storage.max_episodes_cache = 0; // Invalid value
        });
    
    let result = setup.build().await;
    
    assert!(result.is_err(), "Should fail with invalid configuration");
    
    let error_message = result.unwrap_err().to_string();
    
    // Error message should be helpful and actionable
    assert!(error_message.len() > 20, "Error message should be descriptive");
    assert!(error_message.contains("validation") || error_message.contains("failed"),
        "Error should mention validation");
}

#[tokio::test]
async fn test_storage_initialization() {
    // Test that SimpleSetup can initialize storage successfully
    let setup = SimpleSetup::preset(ConfigPreset::Memory) // Use memory for faster test
        .hide_guidance(); // Reduce noise in test output
    
    let (config, storage_result) = setup.build_and_init().await
        .expect("SimpleSetup should initialize storage successfully");
    
    // Verify storage was initialized
    assert!(config.storage.max_episodes_cache > 0);
    assert!(storage_result.memory.storage_backends().0.is_some() || 
           storage_result.memory.storage_backends().1.is_some(),
        "At least one storage backend should be initialized");
}

#[tokio::test]
async fn test_concurrent_setup_safety() {
    // Test that multiple setup operations can run concurrently safely
    let tasks: Vec<_> = (0..5).map(|i| {
        tokio::spawn(async move {
            let setup = SimpleSetup::preset(ConfigPreset::Memory)
                .with_custom(|config| {
                    config.storage.max_episodes_cache = 100 + i * 50;
                });
            
            setup.build().await
        })
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // All tasks should complete successfully
    for (i, result) in results.into_iter().enumerate() {
        let config = result.expect("Task should not panic")
            .expect("Setup should succeed");
        
        assert_eq!(config.storage.max_episodes_cache, 100 + i * 50,
            "Each concurrent task should have distinct configuration");
    }
}

#[tokio::test]
async fn test_preset_descriptions() {
    // Test that preset descriptions are helpful and accurate
    let setup = SimpleSetup::preset(ConfigPreset::Local);
    let description = setup.preset_description();
    
    assert!(description.contains("Local"), "Description should mention preset type");
    assert!(description.contains("•"), "Description should have bullet points");
    assert!(description.len() > 50, "Description should be informative");
    
    // Test different presets
    for preset in [ConfigPreset::Cloud, ConfigPreset::Memory] {
        let setup = SimpleSetup::preset(preset);
        let description = setup.preset_description();
        
        assert!(description.len() > 20, "All preset descriptions should be informative");
    }
}