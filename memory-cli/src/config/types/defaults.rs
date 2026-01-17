//! Smart defaults for configuration based on environment detection

use std::env;
use std::path::PathBuf;
use sysinfo::System;

use super::SystemInfo;

/// Detect the appropriate data directory based on platform and environment
pub fn detect_data_directory() -> PathBuf {
    // Check environment variable first
    if let Ok(data_dir) = env::var("MEMORY_DATA_DIR") {
        return PathBuf::from(data_dir);
    }

    // Platform-appropriate default directories
    if let Some(mut data_dir) = dirs::data_dir() {
        data_dir.push("memory-cli");
        data_dir
    } else if let Some(mut home_dir) = dirs::home_dir() {
        home_dir.push(".memory-cli");
        home_dir
    } else {
        // Fallback to current directory
        PathBuf::from("./data")
    }
}

/// Detect cache directory for redb storage
pub fn detect_cache_directory() -> PathBuf {
    // Check environment variable first
    if let Ok(cache_dir) = env::var("MEMORY_CACHE_DIR") {
        return PathBuf::from(cache_dir);
    }

    let mut data_dir = detect_data_directory();
    data_dir.push("cache");
    data_dir
}

/// Get system resource information for smart configuration
pub fn get_system_info() -> SystemInfo {
    let mut system = System::new_all();
    system.refresh_all();

    SystemInfo {
        total_memory: system.total_memory(),
        available_memory: system.available_memory(),
        cpu_count: System::physical_core_count().unwrap_or(1),
        is_ci: env::var("CI").is_ok(),
        is_development: env::var("DEVELOPMENT").is_ok() || env::var("DEV").is_ok(),
    }
}

/// Smart database path detection
pub fn detect_redb_path() -> String {
    // Check environment variable
    if let Ok(path) = env::var("REDB_PATH") {
        return path;
    }

    // Use cache directory with memory-specific filename
    let mut cache_dir = detect_cache_directory();
    cache_dir.push("memory.redb");
    cache_dir.to_string_lossy().to_string()
}

/// Smart pool size based on system resources
pub fn suggest_pool_size() -> usize {
    let info = get_system_info();

    if info.is_ci || info.is_development {
        5 // Conservative for CI/dev
    } else {
        // Scale with CPU cores, max 20, min 3
        std::cmp::max(3, std::cmp::min(20, info.cpu_count * 2))
    }
}

/// Smart cache size based on available memory
pub fn suggest_cache_size() -> usize {
    let info = get_system_info();

    if info.is_ci {
        100 // Minimal for CI
    } else if info.is_development {
        500 // Moderate for development
    } else {
        // Scale with available memory: 1GB = ~200 episodes
        let gb_available = info.available_memory / (1024 * 1024 * 1024);
        std::cmp::min(5000, std::cmp::max(1000, (gb_available * 200) as usize))
    }
}

/// Smart cache TTL based on system and usage patterns
pub fn suggest_cache_ttl() -> u64 {
    let info = get_system_info();

    if info.is_ci {
        300 // 5 minutes for CI
    } else if info.is_development {
        1800 // 30 minutes for development
    } else {
        7200 // 2 hours for production
    }
}

/// Detect default output format based on environment
pub fn detect_default_format() -> String {
    if let Ok(format) = env::var("MEMORY_FORMAT") {
        return format;
    }

    if env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok() {
        return "json".to_string(); // Machine-readable for CI
    }

    "human".to_string() // Human-readable by default
}

/// Smart batch size for operations
pub fn suggest_batch_size() -> usize {
    let info = get_system_info();

    if info.is_ci {
        10 // Small batches for CI
    } else if info.is_development {
        50 // Moderate for development
    } else {
        200 // Larger for production
    }
}

/// Check if running in cloud environment
pub fn is_cloud_environment() -> bool {
    env::var("TURSO_URL").is_ok() && env::var("TURSO_TOKEN").is_ok()
}

/// Smart Turso URL detection
pub fn detect_turso_url() -> Option<String> {
    if let Ok(url) = env::var("TURSO_URL") {
        return Some(url);
    }

    // Check for common cloud indicators
    if env::var("RENDER").is_ok() || env::var("HEROKU").is_ok() || env::var("FLY_IO").is_ok() {
        return Some("file:./data/memory.db".to_string());
    }

    None
}

/// Smart Turso token detection
pub fn detect_turso_token() -> Option<String> {
    if let Ok(token) = env::var("TURSO_TOKEN") {
        return Some(token);
    }
    None
}
