#!/usr/bin/env bash
# check-docs-integrity.sh - Validate markdown links, script references, and core doc version sync.

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CHECK_URLS="false"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--check-urls]

Options:
  --check-urls   Also verify external https links with HEAD requests (slow)
EOF
}

for arg in "$@"; do
  case "$arg" in
    --check-urls)
      CHECK_URLS="true"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $arg" >&2
      usage >&2
      exit 1
      ;;
  esac
done

cd "$PROJECT_ROOT"

if [[ ! -f "Cargo.toml" ]]; then
  echo "Cargo.toml not found at repository root" >&2
  exit 1
fi

link_failures=0
script_failures=0
version_failures=0

echo "[docs-integrity] Checking markdown links..."

python3 - <<'PY'
import os
import re
import subprocess
import sys

repo = os.getcwd()
link_re = re.compile(r"\[[^\]]+\]\(([^)]+)\)")

files = subprocess.check_output(["git", "ls-files", "*.md"], text=True).splitlines()
broken = []

for rel_path in files:
    abs_path = os.path.join(repo, rel_path)
    base_dir = os.path.dirname(abs_path)
    try:
        with open(abs_path, "r", encoding="utf-8") as f:
            for idx, line in enumerate(f, start=1):
                for raw_target in link_re.findall(line):
                    target = raw_target.strip()
                    if not target:
                        continue
                    if target.startswith(("http://", "https://", "mailto:", "#")):
                        continue
                    if target.startswith("<") and target.endswith(">"):
                        target = target[1:-1].strip()
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
link_status=$?
if [[ $link_status -ne 0 ]]; then
  link_failures=1
  python3 - <<'PY'
import os
import re
import subprocess

repo = os.getcwd()
link_re = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
files = subprocess.check_output(["git", "ls-files", "*.md"], text=True).splitlines()
for rel_path in files:
    abs_path = os.path.join(repo, rel_path)
    base_dir = os.path.dirname(abs_path)
    try:
        with open(abs_path, "r", encoding="utf-8") as f:
            for idx, line in enumerate(f, start=1):
                for raw_target in link_re.findall(line):
                    target = raw_target.strip()
                    if not target or target.startswith(("http://", "https://", "mailto:", "#")):
                        continue
                    if target.startswith("<") and target.endswith(">"):
                        target = target[1:-1].strip()
                    target = target.split("#", 1)[0]
                    if not target:
                        continue
                    resolved = os.path.normpath(os.path.join(base_dir, target))
                    if not os.path.exists(resolved):
                        print(f"  - {rel_path}:{idx} -> {raw_target}")
    except UnicodeDecodeError:
        continue
PY
else
  echo "[docs-integrity] Markdown link check passed"
fi

echo "[docs-integrity] Checking script references in markdown..."
while IFS=: read -r file _line _match; do
  ref=$(echo "$file:${_line}:${_match}" | awk -F: '{print $3}')
  if [[ -n "$ref" && ! -e "$ref" ]]; then
    echo "  - Missing script reference: $file:${_line} -> $ref"
    script_failures=1
  fi
done < <(rg -n --no-heading --color never "scripts/[A-Za-z0-9_./-]+\.sh" --glob "*.md" || true)

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

if [[ $link_failures -ne 0 || $script_failures -ne 0 || $version_failures -ne 0 ]]; then
  echo "[docs-integrity] FAILED"
  exit 1
fi

echo "[docs-integrity] All checks passed"
