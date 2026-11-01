//! Setup commands must remain usable when configuration or storage is broken.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cli(dir: &TempDir, config: &std::path::Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("do-memory-cli"));
    command.env("RUST_LOG", "info");
    command.current_dir(dir.path()).arg("--config").arg(config);
    command
}

#[test]
fn completions_ignore_missing_or_malformed_config() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let config = dir.path().join("broken.toml");

    for content in [None, Some("invalid = [")] {
        if let Some(content) = content {
            std::fs::write(&config, content)?;
        }
        for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
            cli(&dir, &config)
                .args(["completion", shell])
                .assert()
                .success()
                .stdout(predicate::str::contains("do-memory-cli"));
        }
        cli(&dir, &config)
            .args(["episode", "list"])
            .assert()
            .failure();
    }
    Ok(())
}

#[test]
fn template_is_valid_toml_even_with_malformed_config() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let config = dir.path().join("broken.toml");
    std::fs::write(&config, "invalid = [")?;

    let output = cli(&dir, &config)
        .args(["config", "show-template"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let template: toml::Value = toml::from_str(std::str::from_utf8(&output)?)?;
    assert_eq!(template["database"]["storage_mode"].as_str(), Some("local"));
    Ok(())
}

#[test]
fn init_recovers_from_broken_config_without_overwriting_it() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let config = dir.path().join("broken.toml");
    std::fs::write(&config, "invalid = [")?;
    let destination = dir.path().join("new.toml");

    cli(&dir, &config)
        .args(["config", "init"])
        .arg(&destination)
        .assert()
        .success();
    let content = std::fs::read_to_string(&destination)?;
    toml::from_str::<toml::Value>(&content)?;
    cli(&dir, &config)
        .args(["config", "init"])
        .arg(&config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));
    assert_eq!(std::fs::read_to_string(config)?, "invalid = [");
    Ok(())
}

#[test]
fn completions_do_not_open_storage() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let config = dir.path().join("config.toml");
    let database = dir.path().join("data.redb");
    let mut settings = do_memory_cli::config::Config::default();
    settings.database.storage_mode = Some("local".into());
    settings.database.redb_path = Some(database.to_string_lossy().into_owned());
    std::fs::write(&config, toml::to_string(&settings)?)?;

    cli(&dir, &config)
        .args(["completion", "bash"])
        .assert()
        .success();
    assert!(!database.exists(), "completion must not create a database");
    Ok(())
}

#[test]
fn setup_commands_ignore_unavailable_storage() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let config = dir.path().join("config.toml");
    let blocker = dir.path().join("not-a-directory");
    std::fs::write(&blocker, "keep")?;
    let mut settings = do_memory_cli::config::Config::default();
    settings.database.storage_mode = Some("local".into());
    settings.database.redb_path = Some(blocker.join("data.redb").to_string_lossy().into_owned());
    std::fs::write(&config, toml::to_string(&settings)?)?;

    for args in [
        vec!["completion", "bash"],
        vec!["config", "show-template"],
        vec!["config", "init", "new.toml"],
    ] {
        cli(&dir, &config)
            .args(args)
            .assert()
            .success()
            .stderr(predicate::str::contains("Initializing storage backends").not());
    }
    let output = cli(&dir, &config).args(["episode", "list"]).output()?;
    assert!(String::from_utf8(output.stderr)?.contains("Initializing storage backends"));
    assert_eq!(std::fs::read_to_string(blocker)?, "keep");
    Ok(())
}
