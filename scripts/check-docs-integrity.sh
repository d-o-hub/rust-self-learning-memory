#!/usr/bin/env bash
# check-docs-integrity.sh - Validate markdown links, script references, and core doc version sync.

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CHECK_URLS="false"
SCAN_ROOT="$PROJECT_ROOT"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--check-urls] [--root DIR]

Options:
  --check-urls   Also verify external https links with HEAD requests (slow)
  --root DIR     Scan DIR (a git repo) instead of the project root (used by
                 ./scripts/test-docs-integrity.sh fixtures)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check-urls)
      CHECK_URLS="true"
      ;;
    --root)
      [[ $# -ge 2 ]] || { echo "--root needs a directory" >&2; exit 1; }
      SCAN_ROOT="$2"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

cd "$SCAN_ROOT"

if [[ ! -f "Cargo.toml" ]]; then
  echo "Cargo.toml not found at repository root" >&2
  exit 1
fi

link_failures=0
script_failures=0
version_failures=0
fc_failures=0

echo "[docs-integrity] Checking markdown links..."

link_status=0
# `|| link_status=$?` matters: under `set -e` a bare failing heredoc aborted the
# script, so the script-reference, version-sync and fail-closed checks never ran.
python3 - <<'PY' || link_status=$?
import os
import re
import subprocess
import sys

repo = os.getcwd()
link_re = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
code_span_re = re.compile(r"`[^`]*`")

# Skip historical archives: links rot intentionally after consolidation (ADR-039).
SKIP_PREFIXES = (
    "plans/archive/",
    "plans/STATUS/archive/",
)

files = [
    p
    for p in subprocess.check_output(["git", "ls-files", "*.md"], text=True).splitlines()
    if not p.startswith(SKIP_PREFIXES)
]
broken = []

for rel_path in files:
    abs_path = os.path.join(repo, rel_path)
    if not os.path.exists(abs_path):
        continue
    base_dir = os.path.dirname(abs_path)
    try:
        with open(abs_path, "r", encoding="utf-8") as f:
            in_code_block = False
            for idx, line in enumerate(f, start=1):
                stripped = line.strip()
                if stripped.startswith("```"):
                    in_code_block = not in_code_block
                    continue
                if in_code_block:
                    continue
                # A backtick span is quoted literal markdown, not a live link.
                for raw_target in link_re.findall(code_span_re.sub("", line)):
                    target = raw_target.strip()
                    # AGENTS.md mandates <...> around URLs, so the angle brackets
                    # must come off before the external-scheme skip, or every
                    # compliant URL is misread as a broken relative path.
                    if target.startswith("<") and target.endswith(">"):
                        target = target[1:-1].strip()
                    if not target:
                        continue
                    if target.startswith(("http://", "https://", "mailto:", "#")):
                        continue
                    target = target.split("#", 1)[0]
                    if not target:
                        continue
                    resolved = os.path.normpath(os.path.join(base_dir, target))
                    if not os.path.exists(resolved):
                        broken.append((rel_path, idx, raw_target))
    except UnicodeDecodeError:
        # Skip non-utf8 markdown edge cases.
        continue

if broken:
    for rel_path, line, target in broken:
        print(f"BROKEN:{rel_path}:{line}:{target}")
    sys.exit(2)

print("OK")
PY
if [[ $link_status -ne 0 ]]; then
  link_failures=1
else
  echo "[docs-integrity] Markdown link check passed"
fi

echo "[docs-integrity] Checking script references in markdown..."
# Extract bare scripts/*.sh paths (ignore archives; strip backticks/args).
python3 - <<'PY' || script_failures=1
import os
import re
import subprocess
import sys

repo = os.getcwd()
SKIP_PREFIXES = (
    "plans/archive/",
    "plans/STATUS/archive/",
)
# Match scripts/foo.sh or ./scripts/foo.sh inside markdown (not only line starts).
pat = re.compile(r"(?:\./)?(scripts/[A-Za-z0-9_./-]+\.sh)")
files = [
    p
    for p in subprocess.check_output(["git", "ls-files", "*.md"], text=True).splitlines()
    if not p.startswith(SKIP_PREFIXES)
]
missing = []
for rel_path in files:
    abs_path = os.path.join(repo, rel_path)
    if not os.path.exists(abs_path):
        continue
    try:
        with open(abs_path, "r", encoding="utf-8") as f:
            in_code_block = False
            for idx, line in enumerate(f, start=1):
                stripped = line.strip()
                if stripped.startswith("```"):
                    in_code_block = not in_code_block
                    continue
                # Skip fenced examples (ADR snippets often show proposed scripts).
                if in_code_block:
                    continue
                for m in pat.finditer(line):
                    rel_script = m.group(1)
                    if not os.path.exists(os.path.join(repo, rel_script)):
                        missing.append((rel_path, idx, rel_script))
    except UnicodeDecodeError:
        continue

if missing:
    for rel_path, idx, rel_script in missing:
        print(f"  - Missing script reference: {rel_path}:{idx} -> {rel_script}")
    sys.exit(1)
print("OK")
PY

if [[ $script_failures -eq 0 ]]; then
  echo "[docs-integrity] Script reference check passed"
fi

echo "[docs-integrity] Checking core doc version consistency..."
workspace_version=$(grep -E '^version\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"' Cargo.toml | head -1 | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/' || true)
if [[ -z "$workspace_version" ]]; then
  echo "  - Could not detect workspace version from Cargo.toml"
  version_failures=1
else
  for core_doc in README.md AGENTS.md plans/README.md; do
    if [[ -f "$core_doc" ]]; then
      if rg -q "v[0-9]+\.[0-9]+\.[0-9]+" "$core_doc"; then
        if ! rg -q "v${workspace_version}" "$core_doc"; then
          echo "  - Version mismatch in $core_doc (expected v${workspace_version})"
          version_failures=1
        fi
      fi
    fi
  done
fi

echo "[docs-integrity] Checking fail-closed code-execution contract in active docs..."
python3 - <<'PY' || fc_failures=1
import os
import re
import subprocess
import sys

repo = os.getcwd()

# Historical records are exempt: dated ADRs, CHANGELOG, and archived plans.
SKIP_PREFIXES = (
    "plans/archive/",
    "plans/STATUS/archive/",
    "plans/adr/",
    "CHANGELOG.md",
)

# Removed backend/type names that must never be advertised as present.
BANNED = re.compile(
    r"\b(wasmtime-backend|javy-backend|wasm-rquickjs"
    r"|WasmtimeSandbox|UnifiedSandbox|WasmtimeConfig|SandboxBackend)\b"
)

# A banned token is acceptable only when the surrounding text negates it.
NEGATIONS = (
    "no ", "not ", "never", "removed", "absent", "unavailable",
    "does not exist", "do not exist", "without", "nonexistent",
    "historical", "supersed", "obsolete", "rejected", "fail-closed",
)

# Key contract documents must state the fail-closed posture.
REQUIRED = {
    "memory-mcp/README.md": "fail-closed",
    "memory-mcp/SECURITY.md": "fail-closed",
    "docs/API_REFERENCE.md": "fail-closed",
}

files = [
    p
    for p in subprocess.check_output(["git", "ls-files", "*.md"], text=True).splitlines()
    if not p.startswith(SKIP_PREFIXES)
]

problems = []
for rel, marker in REQUIRED.items():
    if not os.path.exists(rel):
        problems.append(f"missing required contract doc: {rel}")
        continue
    with open(rel, encoding="utf-8") as f:
        if marker not in f.read().lower():
            problems.append(f"{rel} does not state the '{marker}' code-execution contract")

for rel in files:
    abs_path = os.path.join(repo, rel)
    if not os.path.exists(abs_path):
        continue
    try:
        with open(abs_path, encoding="utf-8") as f:
            lines = f.read().splitlines()
    except UnicodeDecodeError:
        continue
    for idx, line in enumerate(lines):
        if not BANNED.search(line):
            continue
        # Allow wrapped negations on the next physical line.
        window = (line + " " + (lines[idx + 1] if idx + 1 < len(lines) else "")).lower()
        if not any(n in window for n in NEGATIONS):
            problems.append(f"{rel}:{idx + 1}: advertises removed backend/type: {line.strip()}")

if problems:
    for p in problems:
        print(f"  - {p}")
    sys.exit(1)
print("OK")
PY

if [[ $fc_failures -eq 0 ]]; then
  echo "[docs-integrity] Fail-closed contract check passed"
fi

if [[ "$CHECK_URLS" == "true" ]]; then
  echo "[docs-integrity] Checking external https links (HEAD requests)..."
  python3 - <<'PY'
import os
import re
import subprocess
import sys
import urllib.request

repo = os.getcwd()
link_re = re.compile(r"\[[^\]]+\]\((https://[^)]+)\)")
files = subprocess.check_output(["git", "ls-files", "*.md"], text=True).splitlines()

seen = set()
failed = []
for rel_path in files:
    abs_path = os.path.join(repo, rel_path)
    if not os.path.exists(abs_path):
        continue
    try:
        with open(abs_path, "r", encoding="utf-8") as f:
            for line in f:
                for url in link_re.findall(line):
                    u = url.strip()
                    if u in seen:
                        continue
                    seen.add(u)
                    req = urllib.request.Request(u, method="HEAD")
                    try:
                        with urllib.request.urlopen(req, timeout=8) as resp:
                            if resp.status >= 400:
                                failed.append((u, resp.status))
                    except Exception:
                        failed.append((u, "error"))
    except UnicodeDecodeError:
        continue

if failed:
    for u, status in failed:
        print(f"  - {u} ({status})")
    sys.exit(2)
PY
  if [[ $? -ne 0 ]]; then
    link_failures=1
  fi
fi

if [[ $link_failures -ne 0 || $script_failures -ne 0 || $version_failures -ne 0 || $fc_failures -ne 0 ]]; then
  echo "[docs-integrity] FAILED"
  exit 1
fi

echo "[docs-integrity] All checks passed"
