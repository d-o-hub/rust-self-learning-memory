//! Config template generation (`config show-template` / `config init`).

use anyhow::Context;
use std::path::Path;

use crate::config::Config;

/// Build the starter configuration TOML string.
///
/// Sets `storage_mode = "local"` so the default redb path is used without a
/// remote Turso account. Documented in `docs/CONFIGURATION_GUIDE.md`.
pub fn render_config_template() -> anyhow::Result<String> {
    let mut config = Config::default();
    config.database.storage_mode = Some("local".to_string());
    toml::to_string_pretty(&config).context("Failed to serialize config template")
}

/// Print a starter configuration template (TOML) to stdout.
pub async fn show_config_template() -> anyhow::Result<()> {
    let toml = render_config_template()?;
    println!("{toml}");
    Ok(())
}

/// Write a starter configuration to `path` (default `do-memory-cli.toml`).
///
/// Refuses to overwrite an existing file. The written config uses
/// `storage_mode = "local"` by default so it is immediately usable with redb.
pub async fn init_config(path: &Path) -> anyhow::Result<()> {
    write_config_template(path)
}

/// Synchronous write helper (unit-tested).
pub fn write_config_template(path: &Path) -> anyhow::Result<()> {
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
    let toml = render_config_template()?;
    std::fs::write(path, toml)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;
    println!("✓ Wrote starter configuration to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn render_template_is_valid_toml_with_local_mode() {
        let toml = render_config_template().expect("render");
        assert!(
            toml.contains("storage_mode"),
            "template should set storage_mode"
        );
        let parsed: Config = toml::from_str(&toml).expect("template must parse as Config");
        assert_eq!(parsed.database.storage_mode.as_deref(), Some("local"));
        assert!(parsed.storage.max_episodes_cache > 0);
        assert!(!parsed.cli.default_format.is_empty());
    }

    #[test]
    fn write_config_template_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("do-memory-cli.toml");
        write_config_template(&path).expect("write");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("storage_mode"));
    }

    #[test]
    fn write_config_template_refuses_overwrite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("existing.toml");
        std::fs::write(&path, "already here").unwrap();
        let err = write_config_template(&path).expect_err("must refuse overwrite");
        assert!(
            err.to_string().contains("refusing to overwrite"),
            "got: {err}"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "already here");
    }

    #[test]
    fn write_config_template_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("cfg.toml");
        write_config_template(&path).expect("write nested");
        assert!(path.exists());
    }
}
