//! Storage operations for RedbStorage
//!
//! This module contains the implementation methods for RedbStorage,
//! split into logical submodules:
//!
//! - `schema`: Schema version management and initialization
//! - `migration`: Non-destructive migration and index rebuild
//! - `clear`: Table clearing operations
//! - `stats`: Statistics, health checks, and cache metrics

mod clear;
mod migration;
mod schema;
mod stats;
