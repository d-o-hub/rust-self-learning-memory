# Implementation Process

## Phase 1: Planning

### Understand Requirements
- What is the feature?
- Why is it needed?
- Who will use it?
- Acceptance criteria?

### Design Approach
- How does it fit into existing architecture?
- What modules need changes?
- What new modules are needed?
- Data structures and API signatures?

## Phase 2: Implementation

### Create Module Structure

```bash
# For new module
touch src/new_feature/mod.rs
touch src/new_feature/core.rs
touch src/new_feature/storage.rs

# Update src/lib.rs
# pub mod new_feature;
```

### Structure Template
```
src/
├── new_feature/
│   ├── mod.rs          # Public API exports
│   ├── core.rs         # Core logic (<500 LOC)
│   ├── storage.rs      # Storage operations (<500 LOC)
│   └── types.rs        # Data structures (<500 LOC)
```

## Phase 3: Testing

- Unit tests for core logic
- Integration tests for storage
- Doc tests for public APIs
- Maintain >90% coverage

## Phase 4: Documentation

- Update module docs
- Add examples
- Update AGENTS.md if needed
