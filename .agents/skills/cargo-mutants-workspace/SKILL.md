# Cargo Mutants Workspace Skill

## Context
In a Cargo workspace, when running `cargo-mutants` targeting a specific crate (via `--package`), path parameters given to `--file` options must be relative to the workspace root, not relative to the specific crate directory itself.

## Guidelines
- Always prefix targeted file globs with the sub-crate directory relative to the workspace root.
  - *Bad*: `--file 'src/reward/**/*.rs'`
  - *Good*: `--file 'memory-core/src/reward/**/*.rs'`
- Ensure to verify that targeted files actually exist using an explicit sanity check or file-counting guard before running `cargo mutants`.
