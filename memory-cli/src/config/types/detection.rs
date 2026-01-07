//! Environment detection for automatic configuration preset selection

use std::env;

use super::defaults;
use super::presets::ConfigPreset;
use super::system_info::SystemInfo;

/// Auto-detect the best configuration preset based on environment
pub fn auto_detect_preset(system_info: &SystemInfo) -> ConfigPreset {
    // Priority 1: Check for explicit Turso credentials
    // This takes precedence over other detection methods
    if defaults::is_cloud_environment() {
        tracing::info!(
            "Auto-detected Cloud preset: Turso credentials found (TURSO_URL, TURSO_TOKEN)"
        );
        return ConfigPreset::Cloud;
    }

    // Priority 2: Check for cloud environment indicators
    // This includes platforms like Render, Heroku, Fly.io, etc.
    if env::var("RENDER").is_ok()
        || env::var("HEROKU").is_ok()
        || env::var("FLY_IO").is_ok()
        || env::var("RAILWAY").is_ok()
        || env::var("VERCEL").is_ok()
    {
        tracing::info!("Auto-detected Cloud preset: Cloud platform environment detected");
        return ConfigPreset::Cloud;
    }

    // Priority 3: CI environment gets Memory preset for speed
    if system_info.is_ci {
        tracing::info!("Auto-detected Memory preset: CI environment detected");
        return ConfigPreset::Memory;
    }

    // Priority 4: Default to Local preset for development and general use
    tracing::info!("Auto-detected Local preset: Development/general use environment");
    ConfigPreset::Local
}
