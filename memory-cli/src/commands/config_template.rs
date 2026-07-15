//! Config template generation (`config show-template` / `config init`).

use anyhow::Context;
use std::path::PathBuf;

use crate::config::Config;

/// Print a starter configuration template (TOML) to stdout.
///
/// The template uses a local (redb) backend so it works out of the box
/// without any external service. Mirrors the schema documented in
/// `docs/CONFIGURATION_GUIDE.md`.
pub async fn show_config_template() -> anyhow::Result<()> {
    let mut config = Config::default();
    config.database.storage_mode = Some("local".to_string());
    let toml = toml::to_string_pretty(&config).context("Failed to serialize config template")?;
    println!("{toml}");
    Ok(())
}

/// Write a starter configuration to `path` (default `do-memory-cli.toml`).
///
/// Refuses to overwrite an existing file. The written config uses a local
/// (redb) backend by default so it is immediately usable.
pub async fn init_config(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!(
            "Config file already exists at {}; refusing to overwrite.",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
    }
    let mut config = Config::default();
    config.database.storage_mode = Some("local".to_string());
    let toml = toml::to_string_pretty(&config).context("Failed to serialize config template")?;
    std::fs::write(path, toml)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;
    println!("✓ Wrote starter configuration to {}", path.display());
    Ok(())
}
