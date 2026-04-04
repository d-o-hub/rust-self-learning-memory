//! Vector similarity search operations for Turso
//!
//! This module provides both native vector_top_k (DiskANN-accelerated) and
//! brute-force fallback similarity search for episodes and patterns.

#![allow(unexpected_cfgs)]

mod episodes;
mod patterns;
