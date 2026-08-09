#![no_main]

use libfuzzer_sys::fuzz_target;
use do_memory_core::search::{fuzzy_match, regex_search};
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct SearchInput {
    pattern: String,
    text: String,
    threshold: f64,
}

fuzz_target!(|input: SearchInput| {
    // Fuzz fuzzy match
    let _ = fuzzy_match(&input.pattern, &input.text, input.threshold.clamp(0.0, 1.0));

    // Fuzz regex search
    let _ = regex_search(&input.pattern, &input.text);
});
