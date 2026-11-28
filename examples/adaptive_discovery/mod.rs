//! Building blocks for the adaptive source-discovery example.
//!
//! The example is deliberately fixture-driven so the whole feedback loop fits
//! in one reading:
//!
//! - [`dataset`] loads and validates the deterministic fixture dataset, and owns
//!   the credential-free [`dataset::RedactedEndpoint`] view of a candidate URL.
//! - [`memory_store`] records discovery attempts as episodes and reads them back
//!   after a restart.
//! - [`tags`] owns the episode-tag vocabulary, including the attempt identity that
//!   makes seeding a replay.
//! - [`planner`] turns retrieved evidence into a *bounded* plan decision.
//! - [`metrics`] scores the static baseline and the memory-guided plan against
//!   the fixture ground truth.
//! - [`trace`] emits memory-harness compatible traces so the recorded attempts
//!   can be replayed through `do-memory-cli`.

pub mod dataset;
pub mod memory_store;
pub mod metrics;
pub mod planner;
pub mod tags;
pub mod trace;
