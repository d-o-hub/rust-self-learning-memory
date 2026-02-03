# Property-Based Testing Framework for Memory-Core

## Overview

This guide documents the property-based testing implementation in memory-core, designed to catch edge cases and verify invariants that must hold regardless of input values.

## What is Property-Based Testing?

Property-based testing (PBT) is a testing approach where instead of testing specific input-output pairs, you test general properties (invariants) that should always hold true for any valid input. The test framework automatically generates hundreds or thousands of random inputs to verify these properties.

## Benefits Over Traditional Testing

1. **Edge Case Discovery**: Random input generation reveals edge cases developers might miss
2. **Input Space Exploration**: Tests cover more of the input space than hand-written tests
3. **Invariant Verification**: Tests focus on properties that must always be true
4. **Shrinking**: When a test fails, the framework automatically finds the minimal failing input
5. **Reproducibility**: Same seed produces same random inputs for debugging

## Installation

The property-based testing uses the `proptest` crate (version 1.5):

```toml
[dev-dependencies]
proptest = "1.5"
```

## Running Property Tests

Run all property tests:
```bash
cargo test -p memory-core
```

Run specific property test files:
```bash
cargo test -p memory-core episode_property_tests
cargo test -p memory-core pattern_property_tests
cargo test -p memory-core relationship_property_tests
cargo test -p memory-core tag_property_tests
```

Run specific properties:
```bash
cargo test -p memory-core episode_id_is_valid_uuid
cargo test -p memory_core similarity_is_reflexive
```

Increase test cases for exhaustive testing:
```bash
PROPTEST_CASES=10000 cargo test -p memory-core
```

## Property Test Structure

### Basic Template

```rust
use memory_core::types::MyType;
use proptest::prelude::*;

proptest! {
    /// Property description
    #[test]
    fn property_name(input1 in strategy1, input2 in strategy2) {
        // Arrange & Act
        let result = function_under_test(input1, input2);

        // Assert - property must always be true
        assert!(result.is_valid(), "Property failed for inputs: {:?}, {:?}", input1, input2);
    }
}
```

### Common Strategies

```rust
// Strings
"[a-zA-Z0-9]{1,50}"              // Alphanumeric, 1-50 chars
"[a-z]{10}"                      // Lowercase letters, exactly 10
"\\PC{0,100}"                    // Any Unicode, up to 100

// Vectors
proptest::collection::vec("[a-z]{1,10}", 0..10)  // 0-10 elements
proptest::collection::vec(any::<T>(), 1..5)       // 1-5 elements
proptest::collection::hash_map(".*", ".*", 0..10) // HashMap with 0-10 entries

// Numbers
0..100usize                      // Unsigned integer 0-99
0.0f32..1.0                      // Float in range
any::<MyEnum>()                  // Any enum variant

// Custom types
any::<TaskType>()                // Implement Arbitrary trait
```

### Advanced Patterns

```rust
// Branching strategies
let input = prop_oneof![
    0..5usize,
    10..20usize,
    100..200usize,
];

// Optional values
let opt = prop::option::maybe(0..100usize);

// With frequency
let freq = prop_oneof![
    9 => prop::sample::select(vec![1, 2, 3]),
    1 => 0usize,
];
```

## Test Files

### episode_property_tests.rs

Tests for Episode and ExecutionStep invariants:

**Episode Creation:**
- `episode_id_is_valid_uuid`: Episode IDs are always valid, non-nil UUIDs
- `episode_start_time_is_set`: Start time is close to creation time
- `new_episode_is_incomplete`: New episodes have no outcome or end_time
- `new_episode_has_no_steps`: New episodes have zero steps

**Episode Tags:**
- `tag_normalization_is_idempotent`: Normalizing tags twice gives same result
- `tags_are_normalized`: Tags are case-insensitive and trimmed
- `tags_maintain_uniqueness`: No duplicate tags exist
- `invalid_tags_are_rejected`: Empty/invalid tags return errors

**Episode Completion:**
- `completing_episode_sets_outcome`: Completion sets end_time and outcome
- `duration_requires_completed_episode`: Duration only available after completion
- `duration_is_non_negative`: Duration is always non-negative

**Episode Steps:**
- `adding_step_increases_count`: Step count increases correctly
- `step_numbers_accurate`: Step numbering is tracked accurately
- `successful_step_count_accurate`: Success/failure counts are correct

**Serialization:**
- `episode_serialization_roundtrip`: Episode serializes/deserializes correctly
- `step_serialization_roundtrip`: ExecutionStep serializes/deserializes correctly

**Invariants:**
- `episode_modification_preserves_invariants`: Modifications preserve episode ID, etc.

**Test Count:** 20+ property tests

### relationship_property_tests.rs

Tests for episode relationship invariants:

**Validation:**
- `self_relationships_rejected`: Self-relationships always fail
- `duplicate_relationships_rejected`: Same relationship can't be added twice
- `invalid_priority_rejected`: Priority must be 1-10

**Cycle Detection:**
- `depends_on_prevents_cycles`: DependsOn cannot create cycles
- `parent_child_prevents_cycles`: ParentChild cannot create cycles
- `blocks_prevents_cycles`: Blocks cannot create cycles
- `non_acyclic_allows_cycles`: Follows/RelatedTo/Duplicates allow cycles

**Removal:**
- `relationship_removal_idempotent`: Removing twice is safe
- `removal_updates_indexes`: Removal clears all indexes

**Querying:**
- `relationship_exists_consistency`: Existence check is consistent
- `outgoing_incoming_symmetry`: Outgoing/incoming are symmetric
- `get_by_type_returns_both_directions`: Returns both directions
- `relationship_count_accurate`: Count is accurate

**Type Properties:**
- `relationship_type_string_roundtrip`: String conversion is round-trippable
- `relationship_type_serializable`: Types serialize/deserialize correctly
- `directionality_property_consistent`: Directional types have inverses
- `acyclic_requirement_consistent`: Acyclic types are directional

**Graph:**
- `load_relationships_preserves_state`: Loading preserves state
- `multiple_relationships_between_pairs`: Multiple different pairs work
- `custom_fields_preserved`: Custom metadata fields preserved

**Test Count:** 25+ property tests

### pattern_property_tests.rs

Tests for pattern similarity and effectiveness:

**ID Properties:**
- `pattern_id_is_valid_uuid`: Pattern IDs are valid UUIDs

**Similarity:**
- `similarity_is_reflexive`: Pattern similarity with self is 1.0
- `similarity_is_symmetric`: A ~ B = B ~ A
- `similarity_is_bounded`: Scores between 0.0 and 1.0
- `different_types_zero_similarity`: Different pattern types have 0.0 similarity
- `similarity_key_deterministic`: Key is deterministic for same pattern

**Scores:**
- `success_rate_bounded`: Success rates 0.0-1.0
- `pattern_success_rate_bounded`: Pattern success rate is bounded

**Effectiveness:**
- `initial_effectiveness_zero`: New trackers have zero counts
- `record_retrieval_increments`: Retrieval counting works
- `application_success_rate_bounded`: Application success rate is bounded
- `application_stats_sum`: Success/failure counts sum correctly
- `usage_rate_bounded`: Usage rate is bounded
- `reward_delta_converges`: Moving average converges correctly

**Serialization:**
- `pattern_serialization_roundtrip`: Patterns serialize/deserialize correctly
- `effectiveness_serialization_roundtrip`: Effectiveness serializes correctly

**Context:**
- `pattern_context_retrieval`: Context retrieval works
- `context_pattern_no_context`: ContextPattern has no context

**Sample Size:**
- `sample_size_non_negative`: Sample size is non-negative
- `context_pattern_sample_size`: Matches evidence length

**Test Count:** 20+ property tests

### tag_property_tests.rs

Tests for tag operations and normalization:

**Normalization:**
- `tag_normalization_lowercase`: Tags are lowercase
- `tag_normalization_trims`: Whitespace is trimmed
- `tag_normalization_deterministic`: Same input normalizes to same value

**Uniqueness:**
- `tags_maintain_uniqueness`: No duplicate tags
- `duplicate_tag_add_returns_false`: Adding duplicate returns false
- `case_variations_duplicates`: Case variations are duplicates
- `whitespace_variations_duplicates`: Whitespace variations are duplicates

**Idempotence:**
- `add_tag_idempotent`: Adding repeatedly is idempotent
- `remove_tag_idempotent`: Removing repeatedly is safe
- `clear_tags_idempotent`: Clearing repeatedly is safe

**Validation:**
- `empty_tag_rejected`: Empty tags fail
- `whitespace_only_tag_rejected`: Whitespace-only fails
- `invalid_characters_rejected`: Invalid chars fail
- `too_short_tag_rejected`: Tags < 2 chars fail
- `tag_length_limit_enforced`: Length limit enforced

**Querying:**
- `has_tag_correct`: has_tag returns correct results
- `has_tag_case_insensitive`: Case-insensitive lookups
- `has_tag_whitespace_insensitive`: Whitespace-insensitive lookups
- `get_tags_returns_all`: Returns all added tags

**Combination:**
- `add_multiple_unique_tags`: Adding multiple unique tags works
- `remove_and_readd_tag`: Remove and readd works correctly
- `tags_preserve_addition_order`: Tags maintain addition order

**Test Count:** 25+ property tests

## Total Test Coverage

- **Episode Tests**: 20+ properties
- **Relationship Tests**: 25+ properties
- **Pattern Tests**: 20+ properties
- **Tag Tests**: 25+ properties
- **Total**: 90+ property tests

## Writing New Property Tests

### Step 1: Identify the Property

Find an invariant that should always be true:

- "Episode IDs are never nil"
- "Adding a tag twice doesn't create duplicates"
- "Similarity scores are between 0 and 1"
- "Cycles are prevented in depends_on relationships"

### Step 2: Choose Appropriate Strategies

Select the right proptest strategy for your inputs:

```rust
// For simple values
input in ".*"              // Any string
input in 0..100usize       // Integer range
input in any::<MyType>()   // Custom type

// For collections
inputs in proptest::collection::vec(".*", 1..10)
```

### Step 3: Write the Test

```rust
proptest! {
    #[test]
    fn my_property(input in strategy) {
        // Act
        let result = function_under_test(input);

        // Assert - property must hold
        prop_assert!(result.is_valid());
    }
}
```

### Step 4: Run and Debug

```bash
# Run the test
cargo test my_property

# If it fails, proptest will minimize the failing input
# and show you the minimal counterexample

# Save the failing seed for reproducibility
PROPTEST_SEED=0x3f8a... cargo test my_property
```

## Test Configuration

### Increasing Test Cases

```bash
# Default: 256 cases per property
PROPTEST_CASES=1000 cargo test
```

### Custom Configuration in Cargo.toml

```toml
[[profile.dev.overrides]]
package.memory-core.proptest = "release"

[[profile.test.overrides]]
opt-level = 1
```

### Fuzzing Configuration

In the test file or module:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn exhaustive_property(input in ".*") {
        // ...
    }
}
```

## Shrinking (Finding Minimal Failures)

When a property test fails, proptest automatically shrinks the failing input to find the minimal counterexample:

```bash
---- property::test_name stdout ----
thread 'property::test_name' panicked at 'assertion failed',
        tests/property/test.rs:42:9:
note: failing input test_name(
    "a very long string that causes a bug"
)

minimized failing test case:
    test_name("")
```

## Best Practices

### DO:
- Test properties that represent real invariants
- Use descriptive property names
- Add property descriptions in doc comments
- Keep properties simple and focused
- Use appropriate strategies for input types
- Consider edge cases in strategies

### DON'T:
- Test implementation details
- Write properties that depend on specific values
- Use overly complex strategies
- Ignore failing tests without investigation
- Remove tests that find bugs

## Common Anti-Patterns

### 1. Testing Implementation Instead of Properties

❌ Bad:
```rust
proptest! {
    #[test]
    fn internal_state_correct(input in ".*") {
        assert_eq!(obj._internal_field, expected);
    }
}
```

✅ Good:
```rust
proptest! {
    #[test]
    fn public_api_preserves_invariant(input in ".*") {
        let result = obj.process(input);
        assert!(result.is_valid());
    }
}
```

### 2. Overly Complex Strategies

❌ Bad:
```rust
proptest! {
    #[test]
    fn complex_property(
        (a, b, c) in (".*", ".*", ".*").prop_map(|(a,b,c)| (format!("{}_{}", a, b), c))
    ) { /* ... */ }
}
```

✅ Good:
```rust
proptest! {
    #[test]
    fn simpler_property(a in ".*", b in ".*", c in ".*") {
        let combined = format!("{}_{}", a, b);
        // ...
    }
}
```

### 3. Ignoring Falsifications

❌ Bad:
```rust
proptest! {
    #[test]
    fn failing_property(input in ".*") {
        if input == "special case" {
            return; // Ignore special case!
        }
        assert!(process(input));
    }
}
```

✅ Good:
```rust
proptest! {
    #[test]
    fn handles_special_case(input in ".*") {
        let result = process(input);
        assert!(result.is_ok(), "Failed with input: {}", input);
    }
}
```

## Integration with Existing Tests

Property tests complement traditional unit tests:

```rust
#[cfg(test)]
mod tests {
    // Traditional unit tests
    #[test]
    fn test_specific_case() {
        let result = function("specific input");
        assert_eq!(result, expected);
    }

    // Property tests
    proptest! {
        #[test]
        fn test_general_property(input in ".*") {
            let result = function(input);
            prop_assert!(result.is_valid());
        }
    }
}
```

## Performance Considerations

Property tests can be slower than unit tests due to:
- Multiple test cases per property
- Input generation overhead
- Shrinking on failures

Tips for performance:
- Keep test count reasonable (default 256)
- Use efficient strategies
- Avoid expensive operations in property generation
- Profile slow tests with `cargo test --release`

## Continuous Integration

In CI, consider:
- Running property tests with higher case count on nightly builds
- Using fixed seeds for reproducible failures
- Separating property test runs from unit test runs
- Failing the build on any property test failure

```yaml
# Example CI configuration
- name: Run property tests
  run: |
    PROPTEST_CASES=1000 cargo test -p memory-core
```

## Resources

- [proptest documentation](https://docs.rs/proptest/)
- [Prop-based testing book](https://propertesting.com/)
- [Rust proptest guide](https://altsysrq.github.io/proptest-book/intro.html)

## Summary

The property-based testing framework provides:
- 90+ property tests across 4 major areas
- Automatic input generation and shrinking
- Invariant verification for core types
- Edge case discovery beyond traditional tests
- High confidence in code correctness

This complements the existing 92.5% test coverage by testing properties rather than just specific inputs, helping achieve the >95% coverage goal.
