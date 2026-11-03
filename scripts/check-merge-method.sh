#!/usr/bin/env bash
# Merge Method and No-Bypass Contract Gate
#
# WHY (LESSON-081): ruleset `main-protection` (id 9591004) carries
# `required_linear_history`, so GitHub refuses a merge commit onto `main` - and it
# refuses it *while executing*. A `--method merge` attempt on a stacked PR first
# fails GraphQL with `part of a stack`; the documented asynchronous retry answers
# `202 status=pending` ("accepted for evaluation"), and only the returned uuid
# reports `Repository rule violations found / This branch must not contain merge
# commits.` That cost a full merge round-trip on PR #1241. This gate asserts, before
# anything is merged, that the sanctioned script's default method is one the live
# ruleset can accept.
#
# It also pins the safety property the repository's own instructions rest on:
# `merge-pr.sh` must never request a ruleset bypass. A comment saying "never
# bypass_rules" is not enforcement - the flag has to stay out of the request.
#
# The live ruleset check follows the pattern established in
# scripts/validate-gate-contract.sh:215-235 - fail-closed when a token is present
# (CI), opportunistic with a NOTE when it is not (local), so the gate can neither
# silently no-op in CI nor break a tokenless workstation.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SCRIPT="scripts/merge-pr.sh"
RULESET_ID=9591004

fail() {
    echo "HARNESS VIOLATION: merge-method - $1" >&2
    exit 1
}

echo "=== Merge Method and No-Bypass Contract ==="

[ -f "$SCRIPT" ] || fail "$SCRIPT is missing; the sanctioned merge path no longer exists"

# 1. The default method must be parseable, and must be a method that survives a
#    linear-history rule. `merge` is the value that produced the LESSON-081
#    round-trip, so it is rejected here whatever the ruleset says: re-enabling it
#    means re-reading that lesson first.
METHOD_LINE="$(grep -m1 -E '^METHOD="' "$SCRIPT" || true)"
[ -n "$METHOD_LINE" ] || fail "cannot parse the default METHOD from $SCRIPT"
METHOD="${METHOD_LINE#METHOD=\"}"
METHOD="${METHOD%%\"*}"
case "$METHOD" in
    squash | rebase) echo "OK: $SCRIPT defaults to --method $METHOD (linear history)" ;;
    merge) fail "$SCRIPT defaults to \`merge\`, which ruleset $RULESET_ID's required_linear_history refuses on \`main\`; use squash (rebase only for branches with no merge commits) - LESSON-081" ;;
    *) fail "$SCRIPT has an unrecognised default METHOD='$METHOD'" ;;
esac

# 2. No bypass may ever be requested. Whole-line comments are stripped first: this
#    file and the script both *describe* the prohibition, and a matcher that reads
#    those mentions as violations makes the guard useless (LESSON-079).
if grep -vE '^[[:space:]]*#' "$SCRIPT" | grep -qE -- '-[fF][[:space:]]+bypass_rules|bypass_rules[[:space:]]*[:=][[:space:]]*true'; then
    fail "$SCRIPT passes bypass_rules to the merge API; the repository forbids any bypass - LESSON-081"
fi
echo "OK: $SCRIPT never requests a ruleset bypass"

# 3. A stacked PR is refused by the synchronous endpoint, so the asynchronous API
#    must be wired to the head SHA the gates verified. Without the pin, a push that
#    lands between the gate and the merge would merge an unverified commit.
grep -q 'merge-async' "$SCRIPT" ||
    fail "$SCRIPT lost its stacked-PR path; GitHub refuses \`gh pr merge\` for a PR in a stack - LESSON-081"
# shellcheck disable=SC2016  # '$HEAD_SHA' is literal source text to match, not a variable to expand
grep -qE -- '-f sha="\$HEAD_SHA"' "$SCRIPT" ||
    fail "$SCRIPT's asynchronous merge is no longer pinned to the verified \$HEAD_SHA"
echo "OK: stacked-PR merge is asynchronous and pinned to the verified head SHA"

# 4. Cross-check the assumption against the live ruleset.
RULESET_QUERY='.rules[] | select(.type=="required_linear_history") | .type'
ALLOWED_QUERY='.rules[] | select(.type=="pull_request") | .parameters.allowed_merge_methods[]'
if [[ -n "${GH_TOKEN:-}${GITHUB_TOKEN:-}" ]]; then
    command -v gh >/dev/null 2>&1 ||
        fail "a token is set but gh is unavailable; the live ruleset cannot be verified"
    if ! live_linear=$(gh api "repos/{owner}/{repo}/rulesets/$RULESET_ID" --jq "$RULESET_QUERY" 2>/dev/null); then
        fail "authenticated but unable to read ruleset $RULESET_ID; fix token permissions instead of letting this gate skip"
    fi
    if [[ "$METHOD" == "merge" && -n "$live_linear" ]]; then
        fail "live ruleset $RULESET_ID requires linear history while $SCRIPT defaults to merge"
    fi
    if ! live_allowed=$(gh api "repos/{owner}/{repo}/rulesets/$RULESET_ID" --jq "$ALLOWED_QUERY" 2>/dev/null); then
        fail "cannot read allowed_merge_methods from ruleset $RULESET_ID"
    fi
    if ! printf '%s\n' "$live_allowed" | grep -qx "$METHOD"; then
        fail "live ruleset $RULESET_ID does not list '$METHOD' among allowed merge methods: $(printf '%s ' "$live_allowed")"
    fi
    echo "OK: live ruleset $RULESET_ID allows '$METHOD'$(
        [ -n "$live_linear" ] && echo " with required_linear_history active"
    )"
elif command -v gh >/dev/null 2>&1 &&
    live_allowed=$(gh api "repos/{owner}/{repo}/rulesets/$RULESET_ID" --jq "$ALLOWED_QUERY" 2>/dev/null); then
    if ! printf '%s\n' "$live_allowed" | grep -qx "$METHOD"; then
        fail "live ruleset $RULESET_ID does not list '$METHOD' among allowed merge methods: $(printf '%s ' "$live_allowed")"
    fi
    echo "OK: live ruleset $RULESET_ID allows '$METHOD' (tokenless read)"
else
    echo "NOTE: gh ruleset API unavailable without a token; live cross-check skipped (checks 1-3 still enforced)"
fi

echo "=== Merge Method and No-Bypass Contract Passed ==="
