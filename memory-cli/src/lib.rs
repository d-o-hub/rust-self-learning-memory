#![deny(unsafe_code)]
#![expect(
    clippy::empty_line_after_doc_comments,
    reason = "blank line kept after doc comments"
)]
#![expect(dead_code, reason = "public API unused inside the crate")]
#![expect(unused_variables, reason = "params kept for API symmetry")]
#![expect(unused_mut, reason = "mutation is conditional")]
#![expect(clippy::nonminimal_bool, reason = "boolean expression kept explicit")]
#![expect(clippy::needless_borrow, reason = "borrow kept for uniform call sites")]
#![expect(clippy::manual_clamp, reason = "explicit bounds clearer than clamp")]
#![expect(clippy::derivable_impls, reason = "hand-written impls document intent")]
#![expect(clippy::excessive_nesting, reason = "deeply nested control flow")]
#![expect(
    clippy::if_same_then_else,
    reason = "branches kept separate for clarity"
)]
#![expect(
    clippy::missing_errors_doc,
    reason = "error variants documented separately"
)]
#![expect(
    clippy::missing_panics_doc,
    reason = "panic paths documented separately"
)]
#![expect(
    clippy::cognitive_complexity,
    reason = "long-standing complex functions"
)]
#![expect(
    clippy::must_use_candidate,
    reason = "not every public value is must_use"
)]
#![expect(clippy::doc_markdown, reason = "identifier backticks noisy in prose")]
#![cfg_attr(
    test,
    expect(clippy::panic, reason = "panic used for invariant violations")
)]
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "glob imports used for preludes")
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "precision loss accepted in metric math"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "integer narrowing bounded by checks"
)]
#![expect(
    clippy::cast_sign_loss,
    reason = "non-negative values cast to unsigned"
)]
#![expect(
    clippy::struct_excessive_bools,
    reason = "config struct expressed as flags"
)]
#![expect(clippy::match_same_arms, reason = "explicit arms aid maintenance")]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "explicit closures aid readability"
)]
#![expect(clippy::uninlined_format_args, reason = "format args kept explicit")]
#![expect(clippy::expect_used, reason = "infallible expect in known-good paths")]
#![expect(clippy::unwrap_used, reason = "infallible unwrap in known-good paths")]
#![expect(clippy::map_unwrap_or, reason = "explicit pattern clearer than map_or")]
#![expect(
    clippy::needless_pass_by_value,
    reason = "owned params kept for ergonomics"
)]
#![expect(clippy::unused_async, reason = "async kept for API symmetry")]
#![expect(clippy::assigning_clones, reason = "clone_from not always clearer")]
#![expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "extension comparison kept simple"
)]
#![expect(
    clippy::cast_lossless,
    reason = "explicit widening casts document intent"
)]
#![expect(
    clippy::fn_params_excessive_bools,
    reason = "boolean flags are part of the API"
)]
#![expect(
    clippy::format_push_string,
    reason = "push_str formatting kept explicit"
)]
#![expect(clippy::if_not_else, reason = "positive-first form reads better")]
#![expect(
    clippy::items_after_statements,
    reason = "helpers declared next to use"
)]
#![expect(clippy::manual_let_else, reason = "match kept for clarity")]
#![expect(
    clippy::manual_string_new,
    reason = "explicit String::new kept for clarity"
)]
#![expect(clippy::needless_continue, reason = "continue kept for explicitness")]
#![expect(clippy::ref_option, reason = "&Option kept for API consistency")]
#![expect(
    clippy::return_self_not_must_use,
    reason = "constructors not marked must_use"
)]
#![expect(clippy::single_char_pattern, reason = "single-char pattern kept")]
#![expect(clippy::single_match_else, reason = "explicit arms aid maintenance")]
#![expect(clippy::unnecessary_semicolon, reason = "semicolon kept for clarity")]
#![expect(clippy::if_then_some_else_none, reason = "explicit if/else clearer")]
#![expect(missing_docs, reason = "public docs still incomplete")]

//! # Memory CLI Library
//!
//! This library provides the core functionality for the memory-cli command-line tool.
//! It includes error handling, test utilities, and command implementations.

pub mod commands;
pub mod config;
pub mod errors;
pub mod output;
pub mod test_utils;
