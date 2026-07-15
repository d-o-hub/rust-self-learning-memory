//! CLI flag / env overrides applied after config load.
//!
//! Extracted for unit testing (issue #830): `Config::default()` always fills
//! `redb_path` with an XDG path, so overrides must always replace it.

use super::types::Config;
use std::path::{Path, PathBuf};

/// Apply `--db-path` / `MEMORY_DB_PATH` to a loaded config.
///
/// Always overrides `redb_path` (never "only if None") because
/// [`Config::default`] pre-fills the XDG cache path.
///
/// - **Default CLI (redb-only)**: both `db_path` and `redb_path` use the user
///   path so `Opening redb database at <path>` matches issue #830 exactly.
/// - **With `turso` feature**: sibling files so SQLite and redb never share a
///   path (`memory.db` → redb `memory.redb`).
///
/// When `storage_mode` is unset and no Turso URL is configured, defaults
/// `storage_mode` to `"local"`.
pub fn apply_db_path_override(config: &mut Config, path: &Path) {
    #[cfg(feature = "turso")]
    {
        let (db_path, redb_path) = sibling_db_and_redb_paths(path);
        config.database.db_path = Some(db_path);
        config.database.redb_path = Some(redb_path);
    }
    #[cfg(not(feature = "turso"))]
    {
        let path_str = path.to_string_lossy().to_string();
        config.database.db_path = Some(path_str.clone());
        config.database.redb_path = Some(path_str);
    }

    if config.database.storage_mode.is_none() && config.database.turso_url.is_none() {
        config.database.storage_mode = Some("local".to_string());
    }
}

/// Derive Turso SQLite + redb paths that never point at the same file.
///
/// | User path       | `db_path` (Turso local) | `redb_path` (redb) |
/// |-----------------|-------------------------|--------------------|
/// | `.../memory.db` | `.../memory.db`         | `.../memory.redb`  |
/// | `.../x.redb`    | `.../x.db`              | `.../x.redb`       |
pub fn sibling_db_and_redb_paths(path: &Path) -> (String, String) {
    let is_redb = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("redb"));

    if is_redb {
        let redb = path.to_string_lossy().to_string();
        let mut sqlite = PathBuf::from(path);
        sqlite.set_extension("db");
        (sqlite.to_string_lossy().to_string(), redb)
    } else {
        let db = path.to_string_lossy().to_string();
        let mut redb = PathBuf::from(path);
        redb.set_extension("redb");
        (db, redb.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Config;

    #[test]
    fn always_overrides_xdg_redb_path_from_default() {
        let mut config = Config::default();
        let default_redb = config.database.redb_path.clone();
        assert!(
            default_redb.is_some(),
            "Config::default() must pre-fill redb_path (issue #830 root cause)"
        );

        let custom = PathBuf::from("/tmp/my-project/memory.db");
        apply_db_path_override(&mut config, &custom);

        #[cfg(not(feature = "turso"))]
        {
            assert_eq!(
                config.database.redb_path.as_deref(),
                Some("/tmp/my-project/memory.db"),
                "redb-only CLI: redb opens the exact user path (issue #830)"
            );
        }
        #[cfg(feature = "turso")]
        {
            assert_eq!(
                config.database.redb_path.as_deref(),
                Some("/tmp/my-project/memory.redb"),
                "turso builds: redb uses sibling .redb"
            );
            assert_eq!(
                config.database.db_path.as_deref(),
                Some("/tmp/my-project/memory.db")
            );
            assert_ne!(config.database.db_path, config.database.redb_path);
        }

        assert_ne!(
            config.database.redb_path, default_redb,
            "override must replace default XDG path"
        );
        assert_eq!(config.database.storage_mode.as_deref(), Some("local"));
    }

    #[test]
    fn sibling_mapping_never_shares_path() {
        let (db, redb) = sibling_db_and_redb_paths(Path::new("/data/memory.db"));
        assert_eq!(db, "/data/memory.db");
        assert_eq!(redb, "/data/memory.redb");
        assert_ne!(db, redb);

        let (db2, redb2) = sibling_db_and_redb_paths(Path::new("/data/cache.redb"));
        assert_eq!(db2, "/data/cache.db");
        assert_eq!(redb2, "/data/cache.redb");
        assert_ne!(db2, redb2);
    }

    #[test]
    fn does_not_force_local_when_turso_url_set() {
        let mut config = Config::default();
        config.database.turso_url = Some("libsql://example.turso.io".to_string());
        config.database.storage_mode = None;
        apply_db_path_override(&mut config, Path::new("/tmp/x.db"));
        assert!(config.database.storage_mode.is_none());
    }
}
