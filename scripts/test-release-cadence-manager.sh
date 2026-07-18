#!/usr/bin/env bash
# Test script for release-cadence-manager skill

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="${SCRIPT_DIR}/.."

echo "=== Release Cadence Manager Integration Test ==="
echo ""

# Test 1: Check if skill files exist
echo "Test 1: Checking skill files..."
if [[ -f "$PROJECT_ROOT/.agents/skills/release-cadence-manager/SKILL.md" ]]; then
  echo "  ✓ SKILL.md exists"
else
  echo "  ✗ SKILL.md not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/release-cadence-manager/methodology.md" ]]; then
  echo "  ✓ methodology.md exists"
else
  echo "  ✗ methodology.md not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/release-cadence-manager/agents.md" ]]; then
  echo "  ✓ agents.md exists"
else
  echo "  ✗ agents.md not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/release-cadence-manager/patterns.md" ]]; then
  echo "  ✓ patterns.md exists"
else
  echo "  ✗ patterns.md not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/release-cadence-manager/integration.md" ]]; then
  echo "  ✓ integration.md exists"
else
  echo "  ✗ integration.md not found"
  exit 1
fi

echo ""

# Test 2: Check if CLI script exists and is executable
echo "Test 2: Checking CLI script..."
if [[ -f "$PROJECT_ROOT/scripts/release-cadence-manager.sh" ]]; then
  echo "  ✓ release-cadence-manager.sh exists"
  if [[ -x "$PROJECT_ROOT/scripts/release-cadence-manager.sh" ]]; then
    echo "  ✓ release-cadence-manager.sh is executable"
  else
    echo "  ✗ release-cadence-manager.sh is not executable"
    exit 1
  fi
else
  echo "  ✗ release-cadence-manager.sh not found"
  exit 1
fi

echo ""

# Test 3: Test CLI commands
echo "Test 3: Testing CLI commands..."

# Test help
echo "  Testing help command..."
if "$PROJECT_ROOT/scripts/release-cadence-manager.sh" help >/dev/null 2>&1; then
  echo "  ✓ help command works"
else
  echo "  ✗ help command failed"
  exit 1
fi

# Test detect
echo "  Testing detect command..."
if "$PROJECT_ROOT/scripts/release-cadence-manager.sh" detect >/dev/null 2>&1; then
  echo "  ✓ detect command works"
else
  echo "  ✗ detect command failed"
  exit 1
fi

# Test status
echo "  Testing status command..."
if "$PROJECT_ROOT/scripts/release-cadence-manager.sh" status >/dev/null 2>&1; then
  echo "  ✓ status command works"
else
  echo "  ✗ status command failed"
  exit 1
fi

# Test validate
echo "  Testing validate command..."
if "$PROJECT_ROOT/scripts/release-cadence-manager.sh" validate >/dev/null 2>&1; then
  echo "  ✓ validate command works"
else
  echo "  ✗ validate command failed"
  exit 1
fi

echo ""

# Test 4: Check AGENTS.md updates
echo "Test 4: Checking AGENTS.md updates..."
if grep -q "release-cadence-manager" "$PROJECT_ROOT/AGENTS.md"; then
  echo "  ✓ AGENTS.md has release-cadence-manager reference"
else
  echo "  ✗ AGENTS.md missing release-cadence-manager reference"
  exit 1
fi

if grep -q "Release Cadence Management" "$PROJECT_ROOT/AGENTS.md"; then
  echo "  ✓ AGENTS.md has Release Cadence Management section"
else
  echo "  ✗ AGENTS.md missing Release Cadence Management section"
  exit 1
fi

echo ""

# Test 5: Check integration with existing skills
echo "Test 5: Checking integration with existing skills..."
if [[ -f "$PROJECT_ROOT/.agents/skills/release-guard/SKILL.md" ]]; then
  echo "  ✓ release-guard skill exists"
else
  echo "  ✗ release-guard skill not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/analysis-swarm/SKILL.md" ]]; then
  echo "  ✓ analysis-swarm skill exists"
else
  echo "  ✗ analysis-swarm skill not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/goap-agent/SKILL.md" ]]; then
  echo "  ✓ goap-agent skill exists"
else
  echo "  ✗ goap-agent skill not found"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/.agents/skills/agent-coordination/SKILL.md" ]]; then
  echo "  ✓ agent-coordination skill exists"
else
  echo "  ✗ agent-coordination skill not found"
  exit 1
fi

echo ""

# Test 6: Check workflow integration
echo "Test 6: Checking workflow integration..."
if [[ -f "$PROJECT_ROOT/.github/workflows/release-drift.yml" ]]; then
  echo "  ✓ release-drift.yml workflow exists"
  if grep -q "release-preparation" "$PROJECT_ROOT/.github/workflows/release-drift.yml"; then
    echo "  ✓ workflow has release-preparation label logic"
  else
    echo "  ✗ workflow missing release-preparation label logic"
    exit 1
  fi
else
  echo "  ✗ release-drift.yml workflow not found"
  exit 1
fi

echo ""

# Test 7: Test resolve command (dry run)
echo "Test 7: Testing resolve command (dry run)..."
echo "  Note: Skipping actual PR modification in test mode"
echo "  To test resolve: ./scripts/release-cadence-manager.sh resolve --pr <PR_NUMBER>"

echo ""

# Test 8: Check documentation
echo "Test 8: Checking documentation..."
if [[ -f "$PROJECT_ROOT/plans/GOAP_RELEASE_CADENCE_MANAGER.md" ]]; then
  echo "  ✓ GOAP plan exists"
else
  echo "  ✗ GOAP plan not found"
  exit 1
fi

echo ""

echo "=== Integration Test Complete ==="
echo ""
echo "Summary:"
echo "  ✓ Skill files created"
echo "  ✓ CLI script created and executable"
echo "  ✓ CLI commands working"
echo "  ✓ AGENTS.md updated"
echo "  ✓ Integration with existing skills verified"
echo "  ✓ Workflow integration verified"
echo "  ✓ Documentation created"
echo ""
echo "Next steps:"
echo "  1. Update .github/workflows/release-drift.yml to use new skill"
echo "  2. Test with real PR"
echo "  3. Document lessons learned"
