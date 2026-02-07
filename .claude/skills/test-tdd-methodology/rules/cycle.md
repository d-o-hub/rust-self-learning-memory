# TDD Cycle Rules

## cycle-write-test-first

Write test before implementation.

**Impact:** 40-90% defect reduction

**Why:**
- Forces thinking about interface first
- Ensures testable design
- Documents expected behavior
- Prevents writing untestable code

**Process:**
1. Think about what the code should do
2. Write test describing that behavior
3. Run test → should FAIL
4. Write minimal code to pass

**Example:**
```rust
// Step 1: Write test (RED phase)
#[test]
fn test_calculator_adds_numbers() {
    let calc = Calculator::new();
    assert_eq!(calc.add(2, 3), 5);
}
// Run: FAIL (Calculator doesn't exist yet)

// Step 2: Write minimal code (GREEN phase)
pub struct Calculator;
impl Calculator {
    pub fn add(&self, a: i32, b: i32) -> i32 { a + b }
}
// Run: PASS
```

## cycle-watch-test-fail

Watch test fail before implementing.

**Why:**
- Proves test is actually testing something
- Catches false positives
- Ensures test logic is correct

**Red Flags:**
- Test passes immediately (test is wrong)
- Wrong error message (assertion issue)
- Wrong failure location (setup issue)

## cycle-minimal-code

Write minimal code to make test pass.

**Principle:**
- Hard-code if needed
- No premature generalization
- Just enough to turn red → green

**Example:**
```rust
// Test: assert_eq!(factorial(5), 120);

// Minimal (hard-coded):
fn factorial(n: u32) -> u32 {
    if n == 5 { 120 } else { 0 }
}

// Next test will force generalization
```

## cycle-see-test-pass

Verify green before refactoring.

**Process:**
1. Run all tests
2. Confirm 100% pass
3. Now safe to refactor
4. Run tests after each change

**Safety Net:**
- Tests passing = behavior preserved
- Can refactor with confidence
- Immediate feedback on mistakes

## cycle-refactor-clean

Refactor with tests passing.

**Rules:**
- Only refactor when green
- Make small changes
- Run tests frequently
- Revert if tests fail

**Refactoring Types:**
- Extract method
- Rename variable
- Remove duplication
- Improve naming
- Simplify logic

## cycle-small-steps

Small increments (2-10 min cycles).

**Guidelines:**
- One behavior per cycle
- If stuck, step back
- If taking > 10 min, test too big
- Commit after each green

**Example Cycles:**
1. Test: add positive numbers → Pass
2. Test: add negative numbers → Pass
3. Test: add zero → Pass
4. Refactor: extract validation
