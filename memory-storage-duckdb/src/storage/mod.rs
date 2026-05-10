use crate::DuckDbStorage;
use do_memory_core::Result;

mod analytics;
mod embeddings;
mod episodes;
mod heuristics;
mod monitoring;
mod patterns;
mod recommendations;
mod relationships;

impl DuckDbStorage {
    /// Load VSS extension if enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the extension cannot be loaded or the database task fails.
    pub async fn load_vss_extension(&self) -> Result<()> {
        #[cfg(feature = "vss")]
        {
            use do_memory_core::Error;
            use std::sync::Arc;
            let conn_arc = Arc::clone(&self.conn);
            tokio::task::spawn_blocking(move || {
                let conn = conn_arc.lock();
                conn.execute("INSTALL vss; LOAD vss;", [])
                    .map_err(|e| Error::Storage(format!("Failed to load VSS extension: {e}")))?;
                Ok::<(), Error>(())
            })
            .await
            .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
            Ok(())
        }
        #[cfg(not(feature = "vss"))]
        {
            Ok(())
        }
    }
}
