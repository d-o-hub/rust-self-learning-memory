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
    use uuid::Uuid;

    /// An episode keyed by its own id, wrapped in the map the retriever looks
    /// hits up in.
    fn episode_entry() -> (Uuid, HashMap<Uuid, Arc<Episode>>) {
        let episode = Episode::new(
            "rust api".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        let id = episode.episode_id;
        let mut episodes = HashMap::new();
        episodes.insert(id, Arc::new(episode));
        (id, episodes)
    }

    /// Index holding one vector for `id`, optionally stamped with a provider.
    fn index_with_vector(identity: Option<&str>, id: Uuid) -> SimpleVectorIndex {
        let mut index = match identity {
            Some(identity) => SimpleVectorIndex::with_provider_identity(identity),
            None => SimpleVectorIndex::new(),
        };
        index.upsert(&id.to_string(), &[1.0, 0.0]).unwrap();
        index
    }

    /// Minimal index that leaves the [`VectorIndex`] provenance hooks at their
    /// defaults: it can neither report nor adopt a provider identity.
    struct HooklessIndex {
        inner: SimpleVectorIndex,
    }

    impl VectorIndex for HooklessIndex {
        fn upsert(&mut self, id: &str, embedding: &[f32]) -> crate::error::Result<()> {
            self.inner.upsert(id, embedding)
        }

        fn remove(&mut self, id: &str) -> crate::error::Result<()> {
            self.inner.remove(id)
        }

        fn search(&self, query: &[f32], top_k: usize) -> crate::error::Result<Vec<VectorHit>> {
            self.inner.search(query, top_k)
        }

        fn save(&self, path: &Path) -> crate::error::Result<()> {
            self.inner.save(path)
        }

        fn len(&self) -> usize {
            self.inner.len()
        }
    }

    #[test]
    fn test_retrieve_queries_index_with_matching_identity() {
        let (id, episodes) = episode_entry();
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(index_with_vector(Some("local:a:4"), id)),
        );

        assert_eq!(retriever.provider_identity().as_deref(), Some("local:a:4"));
        assert!(retriever.index_matches_identity());

        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &TaskContext::default(), episodes, 5)
            .unwrap();
        assert_eq!(hits.len(), 1, "a matching identity must be queried");
    }

    #[test]
    fn test_retrieve_refuses_populated_index_bound_to_another_provider() {
        let (id, episodes) = episode_entry();
        let retriever = SemanticRetriever::with_provider_identity(
            MemoryConfig::default(),
            Box::new(index_with_vector(Some("local:a:4"), id)),
            "openai:text-embedding-3-small:1536",
        );

        assert!(!retriever.index_matches_identity());
        assert_eq!(
            retriever.vector_index.read().len(),
            1,
            "the foreign vectors are still physically present"
        );

        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &TaskContext::default(), episodes, 5)
            .unwrap();
        assert!(
            hits.is_empty(),
            "they must not be queried under a different provider identity"
        );
    }

    #[test]
    fn test_retrieve_queries_unstamped_index_without_constraint() {
        let (id, episodes) = episode_entry();
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(index_with_vector(None, id)),
        );

        assert_eq!(retriever.provider_identity(), None);
        assert!(retriever.index_matches_identity());

        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &TaskContext::default(), episodes, 5)
            .unwrap();
        assert_eq!(
            hits.len(),
            1,
            "an index with no recorded provenance stays queryable"
        );
    }

    #[test]
    fn test_with_provider_identity_treats_blank_identity_as_unconstrained() {
        let (id, episodes) = episode_entry();
        let retriever = SemanticRetriever::with_provider_identity(
            MemoryConfig::default(),
            Box::new(index_with_vector(None, id)),
            "   ",
        );

        assert_eq!(
            retriever.provider_identity(),
            None,
            "a blank identity must not constrain the index"
        );
        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &TaskContext::default(), episodes, 5)
            .unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_reconcile_provider_identity_ignores_blank_identity() {
        let (id, _) = episode_entry();
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(index_with_vector(Some("local:a:4"), id)),
        );

        retriever.reconcile_provider_identity("   ");

        assert_eq!(retriever.provider_identity().as_deref(), Some("local:a:4"));
        assert!(retriever.index_matches_identity());
        assert_eq!(
            retriever.vector_index.read().len(),
            1,
            "a blank identity must not drop vectors"
        );
    }

    #[test]
    fn test_reconcile_provider_identity_keeps_vectors_for_same_provider() {
        let (id, _) = episode_entry();
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(index_with_vector(Some("local:a:4"), id)),
        );

        // Padded but equivalent identity: must be treated as unchanged.
        retriever.reconcile_provider_identity(" local:a:4 ");

        assert_eq!(retriever.provider_identity().as_deref(), Some("local:a:4"));
        assert_eq!(
            retriever.vector_index.read().len(),
            1,
            "an unchanged provider must keep its vectors"
        );
    }

    #[test]
    fn test_upsert_is_skipped_when_index_is_quarantined() {
        let retriever = SemanticRetriever::with_provider_identity(
            MemoryConfig::default(),
            Box::new(SimpleVectorIndex::new()),
            "openai:text-embedding-3-small:1536",
        );

        retriever.upsert("ep", vec![1.0]).unwrap();

        assert_eq!(
            retriever.vector_index.read().len(),
            0,
            "vectors must not be written into an index bound to another provider"
        );
    }

    #[test]
    fn test_hookless_index_is_quarantined_once_an_identity_is_required() {
        let dir = tempfile::tempdir().unwrap();
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(HooklessIndex {
                inner: SimpleVectorIndex::new(),
            }),
        );

        // Without a recorded identity the index is usable and delegates writes.
        assert_eq!(retriever.provider_identity(), None);
        assert!(retriever.index_matches_identity());
        retriever.upsert("ep", vec![1.0, 0.0]).unwrap();
        assert_eq!(retriever.vector_index.read().len(), 1);
        retriever.save(&dir.path().join("ann.json")).unwrap();
        assert!(dir.path().join("ann.json").exists());
        retriever.remove("ep").unwrap();
        assert_eq!(retriever.vector_index.read().len(), 0);

        // The default reconcile hook cannot adopt an identity, so the index is
        // quarantined rather than queried with vectors of unknown provenance.
        retriever.reconcile_provider_identity("local:a:4");
        assert_eq!(retriever.provider_identity().as_deref(), Some("local:a:4"));
        assert!(!retriever.index_matches_identity());

        let hits = retriever
            .retrieve("q", &[1.0, 0.0], &TaskContext::default(), HashMap::new(), 5)
            .unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn test_reconcile_drops_vectors_for_new_provider() {
        let (id, episodes) = episode_entry();
        let retriever = SemanticRetriever::new(
            MemoryConfig::default(),
            Box::new(index_with_vector(Some("local:a:4"), id)),
        );
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
        assert_eq!(
            retriever.vector_index.read().len(),
            0,
            "incomparable vectors must be dropped"
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
