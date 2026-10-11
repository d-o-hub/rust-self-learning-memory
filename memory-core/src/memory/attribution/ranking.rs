//! Feedback-to-ranking adaptation (ADR-082).
//!
//! Attribution feedback is reduced to a per-pattern learned weight that re-ranks
//! pattern recommendations. The weight is a deterministic reduction of the
//! in-process tracker plus capability-gated durable history — the Wilson
//! lower-bound success rate — so the index is idempotent (rebuildable),
//! replacement-safe (the tracker, merged last, is authoritative for
//! latest-feedback-per-session), and rollback-safe (drop-and-rebuild, no
//! destructive journal). After a cold restart the index is a pure function of
//! durable history.
//!
//! [`RankingIndex::from_history`] is the canonical rebuild; a single accepted
//! replacement is applied in place by [`RankingIndex::apply_feedback`], which
//! subtracts the replaced session's recorded contribution and adds the new one
//! so refresh cost no longer scales with total history.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::memory::attribution::types::{RecommendationFeedback, RecommendationSession};
use crate::search::ranking::wilson_lower_bound;
use crate::types::TaskOutcome;

/// Z-score for the Wilson lower bound used as the learned weight (matches episode ranking).
pub const RANKING_WILSON_Z: f64 = 1.96; // == z_scores::CONFIDENCE_95
/// Strength of the learned re-rank term relative to base relevance.
pub const LEARNED_BOOST_SCALE: f32 = 0.25;
/// Candidate-pool overfetch factor used on the recommend path so a boosted
/// pattern can enter the top-N (re-rank runs before truncation).
pub const RECOMMEND_OVERFETCH_FACTOR: usize = 3;

/// Durable per-pattern ranking evidence derived from feedback.
///
/// Evidence is `(applied, succeeded)`: a pattern with no applied feedback
/// carries no learned weight, so exposure alone never boosts ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PatternRankingState {
    /// Times the pattern was applied per feedback (the Wilson sample size).
    pub applied: u64,
    /// Applied with outcome `Success | PartialSuccess`.
    pub succeeded: u64,
}

impl PatternRankingState {
    /// Wilson lower bound of p(success | applied); 0.0 when no applied evidence.
    #[must_use]
    pub fn weight(&self, z: f64) -> f64 {
        wilson_lower_bound(self.succeeded, self.applied, z)
    }
}

/// Evidence one session's latest feedback contributed to the derived counters.
///
/// Retained per session so [`RankingIndex::apply_feedback`] can subtract exactly
/// what a replaced record added — in O(applied pattern ids), independent of
/// total history size — instead of rescanning every session. `applied_pattern_ids`
/// mirrors the feedback verbatim (duplicates included) so add and remove stay
/// symmetric.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionContribution {
    applied_pattern_ids: Vec<String>,
    positive: bool,
}

impl SessionContribution {
    /// Reduce one feedback record to its index contribution.
    fn from_feedback(feedback: &RecommendationFeedback) -> Self {
        Self {
            applied_pattern_ids: feedback.applied_pattern_ids.clone(),
            positive: matches!(
                feedback.outcome,
                TaskOutcome::Success { .. } | TaskOutcome::PartialSuccess { .. }
            ),
        }
    }

    /// Add this contribution's evidence to `inner`.
    fn add_to(&self, inner: &mut HashMap<String, PatternRankingState>) {
        for pid in &self.applied_pattern_ids {
            let state = inner.entry(pid.clone()).or_default();
            state.applied += 1;
            if self.positive {
                state.succeeded += 1;
            }
        }
    }

    /// Whether this contribution can be subtracted from `inner`.
    ///
    /// Checked before any mutation, so a `false` result leaves `inner` untouched
    /// and the caller can safely fall back to a full rebuild. Aborts on the
    /// first counter that would underflow.
    fn can_remove_from(&self, inner: &HashMap<String, PatternRankingState>) -> bool {
        let mut required: HashMap<&str, u64> = HashMap::new();
        for pid in &self.applied_pattern_ids {
            *required.entry(pid.as_str()).or_default() += 1;
        }
        required.iter().all(|(pid, count)| {
            inner.get(*pid).is_some_and(|state| {
                state.applied >= *count && (!self.positive || state.succeeded >= *count)
            })
        })
    }

    /// Subtract this contribution's evidence from `inner`.
    ///
    /// Caller must have validated with [`Self::can_remove_from`]. A pattern whose
    /// evidence drops to zero is removed, so the map matches a rebuild exactly
    /// (a rebuild never materializes a zeroed entry).
    fn remove_from(&self, inner: &mut HashMap<String, PatternRankingState>) {
        for pid in &self.applied_pattern_ids {
            if let Some(state) = inner.get_mut(pid) {
                state.applied -= 1;
                if self.positive {
                    state.succeeded -= 1;
                }
                if state.applied == 0 {
                    inner.remove(pid);
                }
            }
        }
    }
}

/// Derived index of learned pattern weights keyed by pattern id string.
///
/// Keys match `Pattern::id().to_string()` and session `recommended_pattern_ids`.
#[derive(Debug, Clone, Default)]
pub struct RankingIndex {
    inner: HashMap<String, PatternRankingState>,
    /// Latest contribution per session, powering the incremental
    /// [`Self::apply_feedback`] path (ADR-082 latest-feedback-wins).
    contributions: HashMap<Uuid, SessionContribution>,
    /// Monotonic mutation counter. A rebuild only overwrites the index while
    /// this is unchanged, so an incremental update racing the rebuild's history
    /// scan is never clobbered by the rebuild's stale snapshot.
    revision: u64,
}

impl RankingIndex {
    /// Deterministic pure function of history.
    ///
    /// Feedback is reduced to the LATEST per session (map overwrite), so
    /// replacement feedback is naturally honored: storage upserts feedback by
    /// `session_id`, and this last-wins reduction mirrors it.
    ///
    /// This is the canonical cold-start/rebuild path; [`Self::apply_feedback`]
    /// is its incremental counterpart for a single accepted replacement.
    #[must_use]
    pub fn from_history(
        sessions: &[RecommendationSession],
        feedback: &[RecommendationFeedback],
    ) -> Self {
        let sessions_by_id: HashSet<Uuid> = sessions.iter().map(|s| s.session_id).collect();

        let mut fb_by_session: HashMap<Uuid, &RecommendationFeedback> = HashMap::new();
        for f in feedback {
            fb_by_session.insert(f.session_id, f);
        }

        let mut index = Self::default();
        for f in fb_by_session.values() {
            if !sessions_by_id.contains(&f.session_id) {
                continue; // orphan feedback contributes nothing
            }
            let contribution = SessionContribution::from_feedback(f);
            contribution.add_to(&mut index.inner);
            index.contributions.insert(f.session_id, contribution);
        }
        index
    }

    /// Apply one session's latest feedback in place (ADR-082 incremental path).
    ///
    /// Subtracts the session's previously indexed contribution (if any) and adds
    /// the replacement, so update cost is independent of total history size. The
    /// caller must have resolved `feedback.session_id` to a known session: an
    /// unknown session contributes nothing in [`Self::from_history`] (orphan
    /// filtering), so that case belongs on the rebuild fallback.
    ///
    /// Returns `false` when the recorded contribution cannot be subtracted
    /// (index inconsistent); the caller must then rebuild from history. On
    /// `false` the index is unchanged.
    #[must_use]
    pub fn apply_feedback(&mut self, feedback: &RecommendationFeedback) -> bool {
        let replacement = SessionContribution::from_feedback(feedback);
        if let Some(prior) = self.contributions.get(&feedback.session_id) {
            if !prior.can_remove_from(&self.inner) {
                return false;
            }
            prior.remove_from(&mut self.inner);
        }
        replacement.add_to(&mut self.inner);
        self.contributions.insert(feedback.session_id, replacement);
        self.revision = self.revision.wrapping_add(1);
        true
    }

    /// Read-only snapshot of the learned pattern weights.
    ///
    /// Copies only the weight map, not the per-session incremental bookkeeping,
    /// so a snapshot stays proportional to the pattern count rather than the
    /// whole session history. The result is for reading ([`Self::boost`] /
    /// [`Self::len`]); running it back through [`Self::apply_feedback`] would
    /// silently restart from empty history, so only the memory layer's live
    /// index is mutated.
    #[must_use]
    pub(crate) fn read_view(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            ..Self::default()
        }
    }

    /// Monotonic mutation counter (see the field documentation).
    #[must_use]
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    /// Overwrite this index with a rebuild, carrying the mutation counter forward
    /// so a concurrent incremental update is still detected afterwards.
    pub(crate) fn replace_with(&mut self, rebuilt: Self) {
        let revision = self.revision.wrapping_add(1);
        *self = rebuilt;
        self.revision = revision;
    }

    /// Learned boost in `[0,1] * LEARNED_BOOST_SCALE` for `pattern_id`; 0.0 when no evidence.
    #[must_use]
    pub fn boost(&self, pattern_id: &str) -> f32 {
        match self.inner.get(pattern_id) {
            Some(st) => (st.weight(RANKING_WILSON_Z) as f32) * LEARNED_BOOST_SCALE,
            None => 0.0,
        }
    }

    /// Number of patterns with derived ranking evidence.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the index contains no derived evidence.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests;
