#!/usr/bin/env bash
# validate-plans.sh — D3.3 / T0 plan hygiene checks
#
# Usage:
#   ./scripts/validate-plans.sh --active-set
#   ./scripts/validate-plans.sh --version-state
#   ./scripts/validate-plans.sh --release-policy
#   ./scripts/validate-plans.sh --adrs
#   ./scripts/validate-plans.sh --identifiers
#   ./scripts/validate-plans.sh --links
#   ./scripts/validate-plans.sh --package-policy PKG
#   ./scripts/validate-plans.sh --adr-decision ADR-XXX
#   ./scripts/validate-plans.sh --supersession
#   ./scripts/validate-plans.sh --all
#   ./scripts/validate-plans.sh --help

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/validate-plans.sh --active-set
  ./scripts/validate-plans.sh --version-state
  ./scripts/validate-plans.sh --release-policy
  ./scripts/validate-plans.sh --adrs
  ./scripts/validate-plans.sh --identifiers
  ./scripts/validate-plans.sh --links
  ./scripts/validate-plans.sh --package-policy PKG
  ./scripts/validate-plans.sh --adr-decision ADR-XXX
  ./scripts/validate-plans.sh --supersession
  ./scripts/validate-plans.sh --all
  ./scripts/validate-plans.sh --help

Flags may be combined (e.g. --active-set --version-state --identifiers --links).
EOF
}

fail() {
  echo "HARNESS VIOLATION: plans — $1" >&2
  exit 1
}

warn() {
  echo "WARN: plans — $1" >&2
}

note() {
  echo "NOTE: plans — $1"
}

check_active_set() {
  local required=(
    plans/GOALS.md
    plans/ACTIONS.md
    plans/GOAP_STATE.md
    plans/ROADMAPS/ROADMAP_ACTIVE.md
    plans/STATUS/CURRENT.md
    plans/GATE_CONTRACT.md
  )
  for f in "${required[@]}"; do
    [[ -f "$f" ]] || fail "missing canonical plan file: $f"
  done

  # R-G4: warn on excess dated GOAP/analysis files at plans root (ADR-039)
  local dated_count
  dated_count=$(find plans -maxdepth 1 -type f \( -name 'GOAP_*20[0-9][0-9]-*.md' -o -name '*-20[0-9][0-9]-*.md' \) 2>/dev/null | wc -l | tr -d ' ')
  # Allow a small number of active dated analysis files (e.g. current recommendations)
  if [[ "${dated_count}" -gt 5 ]]; then
    warn "excess dated files at plans/ root (${dated_count} > 5); archive completed plans per ADR-039"
  fi
  echo "OK: active-set present (dated_root=${dated_count})"
}

check_version_state() {
  local cargo_ver tag_ver
  cargo_ver=$(rg -n '^version\s*=' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
  tag_ver=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "")
  [[ -n "$cargo_ver" ]] || fail "could not parse workspace version from Cargo.toml"

  # CURRENT.md should mention workspace or released version somewhere
  if ! rg -q "$cargo_ver|0\.[0-9]+\.[0-9]+" plans/STATUS/CURRENT.md; then
    fail "plans/STATUS/CURRENT.md does not mention a semver version"
  fi

  echo "OK: version-state cargo=$cargo_ver latest_tag=${tag_ver:-none}"
}

check_release_policy() {
  # Active skills must not instruct manual gh release create without NEVER
  if [[ -f .agents/skills/release-guard/SKILL.md ]]; then
    rg -q 'release-manager.sh ship --execute' .agents/skills/release-guard/SKILL.md \
      || fail "release-guard missing canonical ship path"
    rg -q 'NEVER' .agents/skills/release-guard/SKILL.md \
      || fail "release-guard missing NEVER for manual release"
  fi
  echo "OK: release-policy"
}

check_adrs() {
  local adr_dir="plans/adr"
  [[ -d "$adr_dir" ]] || fail "missing plans/adr directory"
  local count
  count=$(find "$adr_dir" -maxdepth 1 -type f -name 'ADR-*.md' 2>/dev/null | wc -l | tr -d ' ')
  if [[ "$count" -eq 0 ]]; then
    fail "plans/adr has no ADR-*.md files"
  fi
  # ADR-072 is required for authority matrix (T0.2a)
  if ! find "$adr_dir" -maxdepth 1 -type f -name 'ADR-072*.md' | grep -q .; then
    fail "ADR-072 file missing under plans/adr (required for T0.2a)"
  fi
  echo "OK: adrs count=$count (ADR-072 present)"
}

check_identifiers() {
  local adr_dir="plans/adr"
  [[ -d "$adr_dir" ]] || fail "missing plans/adr directory"
  local count
  count=$(find "$adr_dir" -maxdepth 1 -type f -name 'ADR-*.md' 2>/dev/null | wc -l | tr -d ' ')
  if [[ "$count" -eq 0 ]]; then
    fail "identifiers: empty plans/adr (no ADR-*.md)"
  fi

  # Detect duplicate ADR numbers (e.g. two ADR-025-*.md files)
  local dups
  dups=$(find "$adr_dir" -maxdepth 1 -type f -name 'ADR-*.md' -printf '%f\n' \
    | sed -E 's/^ADR-0*([0-9]+).*/\1/' \
    | sort | uniq -d || true)
  if [[ -n "$dups" ]]; then
    # Soft warn for historical duplicates (025, 054 already known); still report
    while IFS= read -r num; do
      [[ -z "$num" ]] && continue
      warn "duplicate ADR number $num (historical aliases may exist)"
      # Match only the ADR-NNN- prefix (zero-padded to 3 digits common form)
      local padded
      padded=$(printf '%03d' "$num" 2>/dev/null || echo "$num")
      find "$adr_dir" -maxdepth 1 -type f \( -name "ADR-${padded}-*.md" -o -name "ADR-${num}-*.md" \) \
        -printf '  %f\n' 2>/dev/null || true
    done <<<"$dups"
  fi
  echo "OK: identifiers count=$count"
}

check_links() {
  # Soft check that GATE_CONTRACT.md exists (canonical gate matrix link target)
  if [[ ! -f plans/GATE_CONTRACT.md ]]; then
    fail "links: plans/GATE_CONTRACT.md missing"
  fi
  echo "OK: links (GATE_CONTRACT.md present)"
}

check_package_policy() {
  local pkg="${1:-}"
  [[ -n "$pkg" ]] || fail "package-policy requires PKG name (e.g. do-memory-cli)"

  case "$pkg" in
    do-memory-cli)
      # Manifest must exist at memory-cli/Cargo.toml (crate package name do-memory-cli)
      if [[ ! -f memory-cli/Cargo.toml ]]; then
        fail "package-policy $pkg: memory-cli/Cargo.toml missing"
      fi
      if ! rg -q 'name\s*=\s*"do-memory-cli"' memory-cli/Cargo.toml; then
        fail "package-policy $pkg: Cargo.toml package name is not do-memory-cli"
      fi
      # Soft note on publish intent if described in docs/workflows
      local publish_hint=0
      if rg -q 'do-memory-cli' .github/workflows/publish-crates.yml 2>/dev/null; then
        publish_hint=1
      fi
      if [[ "$publish_hint" -eq 1 ]]; then
        echo "OK: package-policy $pkg (manifest present; listed in publish workflow)"
      else
        note "package-policy $pkg: not listed in publish-crates.yml (publication policy = local/binary only or deferred)"
        echo "OK: package-policy $pkg (manifest present)"
      fi
      ;;
    *)
      # Generic: package must appear in workspace Cargo.toml members or have a matching dir
      if ! rg -q "$pkg" Cargo.toml 2>/dev/null \
        && [[ ! -f "${pkg}/Cargo.toml" ]] \
        && [[ ! -f "memory-${pkg#do-memory-}/Cargo.toml" ]]; then
        fail "package-policy $pkg: no matching Cargo.toml found"
      fi
      echo "OK: package-policy $pkg"
      ;;
  esac
}

check_adr_decision() {
  local adr="${1:-}"
  [[ -n "$adr" ]] || fail "adr-decision requires ADR-XXX identifier"

  # Normalize: accept ADR-073 or 073
  local num
  num=$(echo "$adr" | sed -E 's/^ADR-//I')
  local matches
  matches=$(find plans/adr -maxdepth 1 -type f -name "ADR-${num}*.md" 2>/dev/null || true)
  if [[ -z "$matches" ]]; then
    # Don't block if missing — pass with note (per goal: soft)
    note "adr-decision ADR-${num}: no plans/adr file found (non-blocking)"
    echo "OK: adr-decision ADR-${num} (missing file noted)"
    return 0
  fi

  local f
  f=$(echo "$matches" | head -1)
  # Soft: look for Status / Decision markers
  if rg -qi 'status|decision|accepted|rejected|proposed' "$f"; then
    echo "OK: adr-decision ADR-${num} (file present: $(basename "$f"))"
  else
    note "adr-decision ADR-${num}: file present but no status/decision keywords"
    echo "OK: adr-decision ADR-${num} (file present)"
  fi
}

check_supersession() {
  # Soft check ADR-072 if present; optionally note links from related ADRs
  local adr072
  adr072=$(find plans/adr -maxdepth 1 -type f -name 'ADR-072*.md' 2>/dev/null | head -1 || true)
  if [[ -z "$adr072" ]]; then
    note "supersession: ADR-072 not present (soft)"
    echo "OK: supersession (ADR-072 absent — soft pass)"
    return 0
  fi
  echo "OK: supersession (ADR-072 present: $(basename "$adr072"))"

  # Soft: related ADRs should mention ADR-072 when supersession notes expected
  local related=(ADR-039 ADR-034 ADR-045 ADR-058)
  for id in "${related[@]}"; do
    local f
    f=$(find plans/adr -maxdepth 1 -type f -name "${id}*.md" 2>/dev/null | head -1 || true)
    if [[ -n "$f" ]]; then
      if rg -q 'ADR-072' "$f"; then
        echo "  OK: $(basename "$f") links ADR-072"
      else
        note "$(basename "$f") does not yet link ADR-072 (soft)"
      fi
    fi
  done
}

# ── Argument parsing (support combined flags) ─────────────────────────────────
if [[ $# -eq 0 ]]; then
  set -- --all
fi

RAN_ANY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --active-set)
      check_active_set
      RAN_ANY=1
      shift
      ;;
    --version-state)
      check_version_state
      RAN_ANY=1
      shift
      ;;
    --release-policy)
      check_release_policy
      RAN_ANY=1
      shift
      ;;
    --adrs)
      check_adrs
      RAN_ANY=1
      shift
      ;;
    --identifiers)
      check_identifiers
      RAN_ANY=1
      shift
      ;;
    --links)
      check_links
      RAN_ANY=1
      shift
      ;;
    --package-policy)
      check_package_policy "${2:-}"
      RAN_ANY=1
      shift 2
      ;;
    --adr-decision)
      check_adr_decision "${2:-}"
      RAN_ANY=1
      shift 2
      ;;
    --supersession)
      check_supersession
      RAN_ANY=1
      shift
      ;;
    --all)
      check_active_set
      check_version_state
      check_release_policy
      check_adrs
      check_identifiers
      check_links
      check_supersession
      RAN_ANY=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown mode: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$RAN_ANY" -eq 0 ]]; then
  echo "No checks selected" >&2
  usage >&2
  exit 2
fi
