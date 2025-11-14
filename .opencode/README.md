# OpenCode Configuration

This directory contains OpenCode-specific configuration for the Rust self-learning memory project.

## Structure

- `plugin/` - JavaScript/TypeScript plugins that extend OpenCode functionality
- `tool/` - Custom tools available to the LLM during conversations
- `agent/` - Specialized agent configurations (adapted from Claude Code)
- `skills/` - Skill definitions for various development tasks

## Plugins

### security.js
Provides security-focused hooks:
- Prevents reading `.env` files
- Runs security checks on session completion (format, clippy, audit, tests)

### final-check.js
Runs final verification checks when sessions end:
- Builds and tests Rust code if modified
- Checks for Cargo.lock changes

## Tools

### code-review.js
Reviews code for quality, correctness, and standards compliance.

### quality.js
- `runTests`: Run cargo tests with various options
- `checkQuality`: Run format, clippy, and security checks

### build.js
Build the Rust project with various options.

## Usage

These plugins and tools are automatically loaded by OpenCode when working in this project directory.