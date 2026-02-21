# Contributing to rust-self-learning-memory

## Development Workflow

1. **Fork & Clone**
   ```bash
   git clone https://github.com/d-o-hub/rust-self-learning-memory.git
   cd rust-self-learning-memory
   ```

2. **Install Git Hooks** (Required)
   ```bash
   # Configure git to use quality-enforcing hooks
   git config core.hooksPath .githooks
   chmod +x .githooks/*
   ```

   These hooks prevent committing broken code and ensure all releases are tested.
   See [`.githooks/README.md`](.githooks/README.md) for details.

3. **Create Feature Branch**
   ```bash
   git checkout -b feat/your-feature-name
   ```

4. **Make Changes**
   - Keep files ≤500 LOC
   - Follow `rustfmt` and `clippy` rules
   - Add tests for new functionality
   - Use postcard for serialization (not bincode)
   - Use parameterized queries for all SQL operations

5. **Run Tests**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all -- -D warnings
   cargo nextest run --all  # preferred over cargo test
   cargo test --doc --all   # doctests require cargo test
   ```

   **Note**: The pre-commit hook automatically runs these checks, but you can run them manually too.

6. **Commit & Push**
   ```bash
   git commit -m "feat: add episode pattern recognition"
   git push origin feat/your-feature-name
   ```

7. **Create Pull Request**
   - Target `main` branch
   - Fill out PR template
   - Link related issues

## Current Status (v0.1.15)

- **Test Pass Rate**: 99.5% (811+ tests passing)
- **Test Coverage**: 92.5% across all modules
- **Clippy Warnings**: 0 (strictly enforced)
- **Performance**: 10-100x faster than baselines

## Code Conventions

See [Code Conventions](agent_docs/code_conventions.md) for detailed guidance:
- Follow Rust idioms and patterns
- Use `anyhow::Result` for errors
- Async for all I/O operations
- Write descriptive commit messages
- Maintain 90%+ test coverage
- Document public APIs with examples

## Testing Requirements

See [Testing Guide](TESTING.md) for comprehensive testing information:
- **Unit Tests**: Within each module (`#[cfg(test)]`)
- **Integration Tests**: Located in `tests/` directory
- **Coverage**: >90% line coverage, >85% branch coverage
- **All tests** must pass before merging
- Use `test-utils` crate for shared helpers

## Commit Message Format

```
<type>: <short summary>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`

**Module scope** (recommended): `feat(core):`, `fix(storage):`, `perf(mcp):`, `ci(workflows):`

Examples:
```
feat: add semantic episode retrieval

Implements vector-based similarity search for episodes using
memory-embed crate integration.

Closes #42
```

```
fix(storage): resolve redb cache corruption on concurrent writes

Fixed race condition in concurrent cache updates. Added proper
transaction handling and improved error messages.

Fixes #123
```

## Code Review Process

- At least 1 approval required
- All CI checks must pass
- Conversations must be resolved
- Code must pass quality gates:
  - ✅ All tests passing (target: >99%)
  - ✅ Coverage >90% (current: 92.5%)
  - ✅ Zero clippy warnings
  - ✅ Code formatted with `cargo fmt`
  - ✅ Security audit passes
  - ✅ Performance benchmarks within 10% regression

## Feature Development

When adding new features:

1. **Design Phase**:
   - Create issue or discussion to propose the feature
   - Get feedback from maintainers
   - Document the design approach

2. **Implementation Phase**:
   - Create feature branch from main
   - Implement with tests (maintain >90% coverage)
   - Follow existing patterns and conventions
   - Keep files under 500 LOC

3. **Testing Phase**:
   - Add unit tests for new functionality
   - Add integration tests for end-to-end behavior
   - Run full test suite
   - Run quality gates

4. **Documentation Phase**:
   - Update README.md if public-facing
   - Update CHANGELOG.md
   - Add inline documentation to public APIs
   - Update relevant docs if needed

5. **Review Phase**:
   - Create pull request
   - Request review
   - Address feedback
   - Merge after approval

## Breaking Changes

Breaking changes must:
1. Be clearly documented in CHANGELOG.md
2. Include migration instructions
3. Have deprecation period if possible
4. Be discussed with maintainers first
5. Include major version bump

## Quality Standards

### Code Quality
- **Formatting**: 100% `cargo fmt` compliant
- **Linting**: Zero clippy warnings (`-D warnings`)
- **Complexity**: Average cyclomatic complexity < 10
- **Documentation**: All public APIs documented
- **Examples**: Code examples for complex operations

### Testing
- **Unit Tests**: All major functions tested
- **Integration Tests**: End-to-end workflows tested
- **Coverage**: >90% line coverage (current: 92.5%)
- **Pass Rate**: >99% (current: 99.3%)
- **Edge Cases**: Error paths and boundaries tested

### Security
- **Input Validation**: All inputs validated at API boundaries
- **SQL Injection**: Parameterized queries only
- **Secrets**: Never hardcoded, use environment variables
- **Serialization**: Use postcard (safer than bincode)
- **Dependencies**: Only from crates.io, compatible licenses

### Performance
- **Regression**: <10% degradation
- **Benchmarks**: Added for performance-critical code
- **Profiling**: Use when optimizing
- **Documentation**: Document performance characteristics

## Getting Help

- **Documentation**: Check `docs/` and `agent_docs/` folders
- **Issues**: Search existing issues first
- **Discussions**: Use GitHub Discussions for questions
- **Code**: Review existing code for patterns

## Resources

- [Project Overview](AGENTS.md) - Detailed project information
- [Building the Project](agent_docs/building_the_project.md) - Build commands
- [Running Tests](agent_docs/running_tests.md) - Testing strategies
- [Code Conventions](agent_docs/code_conventions.md) - Rust idioms
- [Testing Guide](TESTING.md) - Comprehensive testing guide
- [Security](SECURITY.md) - Security policies and practices
- [Release Engineering](plans/adr/ADR-034-Release-Engineering-Modernization.md) - Release workflow
- [Release Automation](docs/RELEASE_AUTOMATION.md) - Automated release process

