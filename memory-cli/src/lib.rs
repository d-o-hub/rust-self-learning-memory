#![allow(clippy::empty_line_after_doc_comments)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(clippy::ifs_same_cond)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::excessive_nesting)]
#![allow(clippy::if_same_then_else)]

//! # Memory CLI Library
//!
//! This library provides the core functionality for the memory-cli command-line tool.
//! It includes error handling, test utilities, and command implementations.

pub mod commands;
pub mod config;
pub mod errors;
pub mod output;
pub mod test_utils;
