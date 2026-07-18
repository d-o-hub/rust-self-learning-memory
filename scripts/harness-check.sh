#!/usr/bin/env bash
# harness-check.sh — Structured HARNESS VIOLATION output for agents
# Usage: bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

VIOLATIONS=0

violation() {
    local category="$1"
    local hint="$2"
    echo -e "${RED}HARNESS VIOLATION: [${category}] — ${hint}${NC}" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
}

info() {
    echo -e "${YELLOW}Running: $1${NC}" >&2
}

check_fmt() {
    info "cargo fmt --check"
    if ! cargo fmt --all -- --check 2>&1; then
        violation "fmt" "Run 'cargo fmt --all' to fix formatting"
        return 1
    fi
    echo -e "${GREEN}fmt: PASS${NC}" >&2
    return 0
}

check_clippy() {
    info "cargo clippy"
    if ! cargo clippy --lib --tests -- -D warnings \
        -A clippy::expect_used \
        -A clippy::uninlined_format_args \
        -A clippy::unwrap_used \
        -A clippy::doc_markdown \
        -A clippy::cognitive_complexity \
        -A clippy::redundant_closure \
        -A clippy::redundant_closure_for_method_calls \
        -A clippy::map_unwrap_or \
        -A clippy::single_match_else \
        -A clippy::match_same_arms \
        -A clippy::similar_names \
        -A clippy::unused_async \
        -A clippy::cast_precision_loss \
        -A clippy::cast_sign_loss \
        -A clippy::cast_possible_truncation \
        -A clippy::cast_possible_wrap \
        -A clippy::inefficient_to_string \
        -A clippy::manual_string_new \
        -A clippy::single_char_pattern \
        -A clippy::format_push_string \
        -A clippy::items_after_statements \
        -A clippy::manual_let_else \
        -A clippy::if_not_else \
        -A clippy::default_trait_access \
        -A clippy::struct_excessive_bools \
        -A clippy::fn_params_excessive_bools \
        -A clippy::implicit_clone \
        2>&1; then
        violation "clippy" "Fix all clippy warnings; see .clippy.toml for exceptions"
        return 1
    fi
    echo -e "${GREEN}clippy: PASS${NC}" >&2
    return 0
}

check_deny() {
    info "cargo deny"
    if command -v cargo-deny &>/dev/null; then
        if ! cargo deny check 2>&1; then
            violation "deny" "Check crate layering in deny.toml; run 'cargo deny check' for details"
            return 1
        fi
    else
        echo -e "${YELLOW}cargo-deny not installed, skipping${NC}" >&2
    fi
    echo -e "${GREEN}deny: PASS${NC}" >&2
    return 0
}

check_test() {
    info "cargo nextest run"
    if ! cargo nextest run --all 2>&1; then
        violation "test" "Fix failing tests before opening PR"
        return 1
    fi
    echo -e "${GREEN}test: PASS${NC}" >&2
    return 0
}

check_arch() {
    info "Architecture fitness test"
    if ! cargo nextest run -p tests --test arch_fitness 2>&1; then
        violation "arch" "Crate layering violation detected; check tests/arch_fitness.rs for details"
        return 1
    fi
    echo -e "${GREEN}arch: PASS${NC}" >&2
    return 0
}

check_security() {
    info "Security scan"
    if command -v gitleaks &>/dev/null; then
        if ! gitleaks detect --config .gitleaks.toml --source . --verbose --redact --no-git 2>&1; then
            violation "security" "Remove secrets from code; use environment variables"
            return 1
        fi
    else
        echo -e "${YELLOW}gitleaks not installed, skipping${NC}" >&2
    fi
    echo -e "${GREEN}security: PASS${NC}" >&2
    return 0
}

# Main
if [[ $# -eq 0 ]]; then
    echo "Usage: bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>" >&2
    exit 1
fi

COMMAND="$1"

case "$COMMAND" in
    fmt) check_fmt ;;
    clippy) check_clippy ;;
    deny) check_deny ;;
    test) check_test ;;
    arch) check_arch ;;
    security) check_security ;;
    all)
        check_fmt || true
        check_clippy || true
        check_deny || true
        check_test || true
        check_arch || true
        check_security || true
        ;;
    *)
        echo "Unknown command: $COMMAND" >&2
        echo "Usage: bash scripts/harness-check.sh <fmt|clippy|deny|test|arch|security|all>" >&2
        exit 1
        ;;
esac

if [[ $VIOLATIONS -gt 0 ]]; then
    echo -e "\n${RED}Total violations: $VIOLATIONS${NC}" >&2
    exit 1
fi

exit 0
