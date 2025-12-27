# Execution Plan: Improve YAML Lint Workflow

## Objective
- Pin actions to SHAs; enhance annotations with problem matcher; least-privilege permissions.

## Proposed Changes
1) Pin actions
- actions/checkout@<sha>
- actions/setup-python@<sha>
- reviewdog/action-actionlint@<sha>
- xt0rted/actionlint-problem-matcher@<sha> (optional)

2) Annotations and thresholds
- Add problem matcher prior to actionlint step.
- Keep fail_level: error and filter_mode: nofilter.
- Choose reporter:
  - github-check (needs checks: write), or
  - github-pr-review (needs pull-requests: write).

3) Least-privilege permissions
- Workflow-level: contents: read.
- Job-level: checks: write or pull-requests: write only where needed.

4) Optional
- Pin Python minor version (e.g., 3.12); cache pip if desired.

## Validation Plan
- actionlint/yamllint; open a PR editing YAML to see annotations.

## Risks & Mitigations
- Duplicate annotations (matcher + reviewdog) → acceptable or tune filter_mode.
- Permissions too strict → adjust per chosen reporter.

## Rollback
- Revert to tags; remove matcher & job-level permissions if needed.

## Implementation Checklist
- [ ] Pin actions
- [ ] Add problem matcher
- [ ] Set minimal permissions
- [ ] Validate on PR
