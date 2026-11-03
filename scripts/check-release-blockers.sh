#!/usr/bin/env bash
# Pre-release Blocker Gate
#
# WHY: releasing is the one action in this repository that cannot be undone
# quietly - a tag is public, `release.yml` publishes, and the version is written
# into the changelog. The policy is therefore that every open issue is
# implemented and every open PR is merged (or deliberately closed) BEFORE a
# release ships, so nothing still under review gets frozen into a published
# version. This script is that policy made executable.
#
# It is a hard block with no bypass flag on purpose. `release-manager.sh` already
# carries `--skip-local-tests` and `--skip-ci-check` for emergencies; a third
# switch meaning "ignore the tracker" would be the one reached for by habit.
#
# One exclusion is built in and it has to be: the release-drift workflow keeps a
# canonical "Release due" issue open until the release happens, so counting it
# would make the gate impossible to satisfy. Excluded issues are printed rather
# than hidden - a gate that can never pass is worse than no gate, and a reader
# has to be able to see what the gate set aside.

set -euo pipefail

REPO=""
SELF_TEST=false
EXCLUDE_LABEL="release-drift"

usage() {
    cat <<'EOF'
Usage: ./scripts/check-release-blockers.sh [--repo <owner/name>] [--self-test]

Reports every open pull request and every open issue as a release blocker.

  --repo <owner/name>    Target repository (default: the one in this directory).
  --exclude-label <lbl>  Label marking a self-maintaining blocker. Default
                         `release-drift`: the drift workflow keeps that issue
                         open until the release it asks for happens.
  --self-test            Run the built-in fixtures that prove the classifier
                         fires, without touching the network.
  -h, --help             Show this help.

Exit codes: 0 no blockers; 1 blockers present; 2 the tracker could not be read
(a blocker gate that silently skipped would be a way to ship unreviewed work).
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo) REPO="${2:?--repo needs a value}"; shift 2 ;;
        --exclude-label) EXCLUDE_LABEL="${2:?--exclude-label needs a value}"; shift 2 ;;
        --self-test) SELF_TEST=true; shift ;;
        -h | --help) usage; exit 0 ;;
        *)
            echo "Unknown arg: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

# Single decision point: the live run and the fixtures both come through here, so
# testing the gate tests the gate and not a copy of it.
evaluate_blockers() {
    local prs_raw="$1" issues_raw="$2"

    local prs issues excluded
    prs=$(jq -c '[.[] | {number, title, draft, author: .user.login, base: .base.ref}]' <<<"$prs_raw")
    # The /issues endpoint also returns pull requests; counting them twice would
    # make the report lie about what is outstanding.
    issues=$(jq -c --arg lbl "$EXCLUDE_LABEL" '
        [.[] | select(has("pull_request") | not)
              | {number, title, labels: [.labels[]?.name]}
              | select(.labels | index($lbl) | not)]' <<<"$issues_raw")
    excluded=$(jq -c --arg lbl "$EXCLUDE_LABEL" '
        [.[] | select(has("pull_request") | not)
              | {number, title, labels: [.labels[]?.name]}
              | select(.labels | index($lbl))]' <<<"$issues_raw")

    local pr_n issue_n excl_n
    pr_n=$(jq 'length' <<<"$prs")
    issue_n=$(jq 'length' <<<"$issues")
    excl_n=$(jq 'length' <<<"$excluded")

    if [[ "$pr_n" == "0" && "$issue_n" == "0" ]]; then
        echo "OK: no open pull requests and no open issues block this release"
        if [[ "$excl_n" != "0" ]]; then
            echo "   excluded as self-maintaining (not counted): $(jq -r '[.[] | "#\(.number) \(.title)"] | join("; ")' <<<"$excluded")"
        fi
        return 0
    fi

    echo "RELEASE BLOCKERS - clear every one of these before shipping:"
    if [[ "$pr_n" != "0" ]]; then
        echo "  $pr_n open pull request(s) - merge or close each:"
        jq -r '.[] | "    PR #\(.number)  [\(if .draft then "draft" else "open" end)]  \(.title)  (head by \(.author), base \(.base))"' <<<"$prs"
    fi
    if [[ "$issue_n" != "0" ]]; then
        echo "  $issue_n open issue(s) - implement or explicitly close each:"
        jq -r '.[] | "    issue #\(.number)  \(.title)"' <<<"$issues"
    fi
    if [[ "$excl_n" != "0" ]]; then
        echo "  excluded, listed so nothing is hidden: $(jq -r '[.[] | "#\(.number)"] | join(" ")' <<<"$excluded")"
    fi
    echo "HARNESS VIOLATION: release-blockers - release blocked by $pr_n PR(s) and $issue_n issue(s)" >&2
    return 1
}

run_self_test() {
    local failures=0

    expect_exit() {
        local label="$1" expected="$2" prs="$3" issues="$4"
        local got=0
        evaluate_blockers "$prs" "$issues" >/dev/null 2>&1 || got=$?
        if [[ "$got" == "$expected" ]]; then
            echo "OK   $label (exit $got)"
        else
            echo "FAIL $label: expected exit $expected, got $got" >&2
            failures=$((failures + 1))
        fi
    }

    local no_prs='[]'
    local one_pr='[{"number":1245,"title":"chore(deps): remove unused declarations","draft":false,"user":{"login":"app/usehoplite"},"base":{"ref":"main"}}]'
    local draft_pr='[{"number":900,"title":"wip","draft":true,"user":{"login":"someone"},"base":{"ref":"main"}}]'
    local no_issues='[]'
    local plain_issue='[{"number":1246,"title":"fix(harness): retire false-green checks","labels":[]}]'
    # Labels are objects here because that is what `GET /repos/{o}/{r}/issues`
    # returns. A fixture with the wrong shape makes the exclusion branch untestable:
    # it silently stops matching, and the case still "passes" for the wrong reason.
    local drift_issue='[{"number":1236,"title":"Release due: 27 commits","labels":[{"name":"jules"},{"name":"release-drift"}]}]'
    local pr_shaped_issue='[{"number":1247,"title":"looks like a PR","labels":[],"pull_request":{"url":"x"}}]'
    local mixed_issue='[{"number":1236,"title":"Release due","labels":[{"name":"release-drift"}]},{"number":1246,"title":"real work","labels":[{"name":"bug"}]}]'

    echo "=== check-release-blockers self-test ==="
    expect_exit "clean tracker" 0 "$no_prs" "$no_issues"
    expect_exit "open PR blocks" 1 "$one_pr" "$no_issues"
    expect_exit "draft PR blocks too" 1 "$draft_pr" "$no_issues"
    expect_exit "plain issue blocks" 1 "$no_prs" "$plain_issue"
    expect_exit "release-drift issue alone does not block" 0 "$no_prs" "$drift_issue"
    expect_exit "PR rows from /issues are not counted" 0 "$no_prs" "$pr_shaped_issue"
    expect_exit "drift exclusion does not hide a real issue" 1 "$no_prs" "$mixed_issue"

    # The exclusion must be visible, not just effective: a reader has to be able to
    # see which issue was set aside, or the gate looks like it is hiding work.
    local report got=0
    report=$(evaluate_blockers "$no_prs" "$mixed_issue" 2>&1) || got=$?
    if [[ "$got" == "1" ]] \
        && grep -q 'issue #1246' <<<"$report" \
        && ! grep -q 'issue #1236' <<<"$report" \
        && grep -q 'excluded, listed so nothing is hidden: #1236' <<<"$report"; then
        echo "OK   excluded issue is named as excluded, not silently dropped (exit $got)"
    else
        echo "FAIL excluded-issue reporting: exit=$got, report was:" >&2
        printf '%s\n' "$report" >&2
        failures=$((failures + 1))
    fi

    if [[ "$failures" != "0" ]]; then
        echo "HARNESS VIOLATION: release-blockers - $failures self-test case(s) failed" >&2
        return 1
    fi
    echo "=== release-blockers self-test passed (8 cases) ==="
    return 0
}

if [[ "$SELF_TEST" == true ]]; then
    command -v jq >/dev/null 2>&1 || { echo "jq is required" >&2; exit 2; }
    run_self_test
    exit $?
fi

command -v gh >/dev/null 2>&1 || {
    echo "ERROR: gh is required to read the tracker. The release must not proceed on an unverified blocker state - install gh, or run this from an authenticated context." >&2
    exit 2
}
command -v jq >/dev/null 2>&1 || {
    echo "ERROR: jq is required to read the tracker." >&2
    exit 2
}

if [[ -z "$REPO" ]]; then
    REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
fi
if [[ -z "$REPO" ]]; then
    echo "ERROR: cannot determine the repository (run inside the clone, or pass --repo <owner/name>)." >&2
    exit 2
fi

echo "=== Pre-release Blocker Gate ($REPO) ==="

prs_raw="$(gh api "repos/$REPO/pulls?state=open&per_page=100" --paginate 2>&1)" || {
    echo "ERROR: could not read open pull requests from $REPO; refusing to release on an unreadable tracker." >&2
    echo "$prs_raw" >&2
    exit 2
}
issues_raw="$(gh api "repos/$REPO/issues?state=open&per_page=100" --paginate 2>&1)" || {
    echo "ERROR: could not read open issues from $REPO; refusing to release on an unreadable tracker." >&2
    echo "$issues_raw" >&2
    exit 2
}

evaluate_blockers "$prs_raw" "$issues_raw"
