//! Provider-identity handling for the ANN-backed [`SemanticRetriever`].
//!
//! Split out of `semantic_retriever.rs` to honor the 500 LOC invariant.
//!
//! The retriever records which embedding provider produced the vectors held in
//! its index. Activation of a new provider realigns that record and drops
//! incomparable vectors; `retrieve` refuses to query an index whose recorded
//! producer does not match the required identity, so vectors from different
//! providers are never mixed.

use crate::embeddings::VectorIndex;

use super::SemanticRetriever;

impl SemanticRetriever {
    /// Create a retriever whose index is bound to `provider_identity`.
    ///
    /// The identity is recorded for the active provider and is what
    /// [`Self::retrieve`] checks the index against.
    pub fn with_provider_identity(
        config: crate::types::MemoryConfig,
        vector_index: Box<dyn VectorIndex>,
        provider_identity: impl Into<String>,
    ) -> Self {
        let provider_identity = provider_identity.into();
        let provider_identity = provider_identity.trim();
        let index_identity = if provider_identity.is_empty() {
            None
        } else {
            Some(provider_identity.to_string())
        };
        Self {
            config,
            vector_index: parking_lot::RwLock::new(vector_index),
            index_identity: parking_lot::RwLock::new(index_identity),
        }
    }

    /// Provider identity this index may be queried with, if constrained.
    #[must_use]
    pub fn provider_identity(&self) -> Option<String> {
        self.index_identity.read().clone()
    }

    /// Whether the stored index actually belongs to the constrained provider.
    ///
    /// An unconstrained retriever (`index_identity == None`) always matches, so
    /// behaviour is unchanged for indexes that never recorded provenance.
    #[must_use]
    pub fn index_matches_identity(&self) -> bool {
        let expected = self.index_identity.read();
        match expected.as_deref() {
            None => true,
            Some(identity) => self.vector_index.read().provider_identity() == Some(identity),
        }
    }

    /// Realign the index with the provider that will serve future queries.
    ///
    /// When `identity` differs from the recorded provider the stored vectors are
    /// dropped and the new identity is recorded (see
    /// [`VectorIndex::reconcile_provider_identity`]). If the index cannot clear
    /// itself, the new identity is still recorded so [`Self::retrieve`]
    /// quarantines the stale vectors instead of querying them.
    pub fn reconcile_provider_identity(&self, identity: &str) {
        let identity = identity.trim();
        if identity.is_empty() {
            return;
        }
        let already_current = self.index_identity.read().as_deref() == Some(identity);
        if already_current {
            return;
        }
        {
            let mut index = self.vector_index.write();
            index.reconcile_provider_identity(identity);
        }
        // Recorded only after the index has been reconciled, so a concurrent
        // reader can never see the new identity against an unreconciled index.
        *self.index_identity.write() = Some(identity.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::SemanticRetriever;
    use crate::embeddings::{SimpleVectorIndex, VectorHit, VectorIndex};
    use crate::episode::Episode;
    use crate::types::{MemoryConfig, TaskContext, TaskType};
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Index with no recorded provider identity that counts how often it is
    /// queried; used to prove a quarantined index is never searched.
    struct CountingIndex {
        searches: Arc<AtomicUsize>,
        identity: Option<String>,
    }

    impl VectorIndex for CountingIndex {
        fn upsert(&mut self, _id: &str, _embedding: &[f32]) -> crate::error::Result<()> {
            Ok(())
        }

        fn remove(&mut self, _id: &str) -> crate::error::Result<()> {
            Ok(())
        }

        fn search(&self, _query: &[f32], _top_k: usize) -> crate::error::Result<Vec<VectorHit>> {
            self.searches.fetch_add(1, Ordering::SeqCst);
            Ok(Vec::new())
        }

        fn save(&self, _path: &Path) -> crate::error::Result<()> {
            Ok(())
        }

        fn len(&self) -> usize {
            0
        }

        fn provider_identity(&self) -> Option<&str> {
            self.identity.as_deref()
        }
    }

    #[test]
    fn test_retrieve_refuses_index_from_another_provider() {
        let searches = Arc::new(AtomicUsize::new(0));
        let retriever = SemanticRetriever::with_provider_identity(
            MemoryConfig::default(),
            Box::new(CountingIndex {
                searches: Arc::clone(&searches),
                identity: None,
            }),
            "openai:text-embedding-3-small:1536",
        );

        let hits = retriever
            .retrieve("q", &[1.0], &TaskContext::default(), HashMap::new(), 5)
            .unwrap();
        assert!(hits.is_empty());
        assert_eq!(
            searches.load(Ordering::SeqCst),
            0,
            "an index that cannot prove its provider must not be queried"
        );
    }

    #[test]
    fn test_retrieve_queries_unconstrained_index() {
        let searches = Arc::new(AtomicUsize::new(0));
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(CountingIndex {
                searches: Arc::clone(&searches),
                identity: None,
            }),
        );

        let hits = retriever
            .retrieve("q", &[1.0], &TaskContext::default(), HashMap::new(), 5)
            .unwrap();
        assert!(hits.is_empty());
        assert_eq!(
            searches.load(Ordering::SeqCst),
            1,
            "an index with no provenance constraint is queried as before"
        );
    }

    #[test]
    fn test_reconcile_drops_vectors_for_new_provider() {
        let episode = Episode::new(
            "rust api".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        let id = episode.episode_id;
        let mut episodes = HashMap::new();
        episodes.insert(id, Arc::new(episode));

        let mut index = SimpleVectorIndex::with_provider_identity("local:a:4");
        index.upsert(&id.to_string(), &[1.0, 0.0]).unwrap();

        let retriever = SemanticRetriever::new(MemoryConfig::default(), Box::new(index));
        assert_eq!(retriever.provider_identity().as_deref(), Some("local:a:4"));

        let context = TaskContext::default();
        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &context, episodes.clone(), 5)
            .unwrap();
        assert_eq!(hits.len(), 1, "index is queried with its own identity");

        retriever.reconcile_provider_identity("openai:text-embedding-3-small:1536");
        assert_eq!(
            retriever.provider_identity().as_deref(),
            Some("openai:text-embedding-3-small:1536")
        );

        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &context, episodes, 5)
            .unwrap();
        assert!(
            hits.is_empty(),
            "stale vectors from the previous provider must not be returned"
        );
    }
}
