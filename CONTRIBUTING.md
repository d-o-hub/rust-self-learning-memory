# Contributing to rust-self-learning-memory

## Development Workflow

1. **Fork & Clone**
   ```bash
   git clone https://github.com/d-o-hub/rust-self-learning-memory.git
   cd rust-self-learning-memory
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feat/your-feature-name
   ```

3. **Make Changes**
   - Keep files ≤500 LOC
   - Follow `rustfmt` and `clippy` rules
   - Add tests for new functionality

4. **Run Tests**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all -- -D warnings
   cargo test --all
   ```

5. **Commit & Push**
   ```bash
   git commit -m "feat: add episode pattern recognition"
   git push origin feat/your-feature-name
   ```

6. **Create Pull Request**
   - Target `main` branch
   - Fill out PR template
   - Link related issues

## Commit Message Format

```
<type>: <short summary>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`

Example:
```
feat: add semantic episode retrieval

Implements vector-based similarity search for episodes using
memory-embed crate integration.

Closes #42
```

## Code Review Process

- At least 1 approval required
- All CI checks must pass
- Conversations must be resolved
- Maintainers will review within 48 hours
