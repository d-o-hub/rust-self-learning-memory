//! Storage operations for RedbStorage
//!
//! This module contains the implementation methods for RedbStorage,
//! split into logical submodules:
//!
//! - `schema`: Schema version management and initialization
//! - `clear`: Table clearing operations
//! - `stats`: Statistics, health checks, and cache metrics

mod clear;
mod schema;
mod stats;
