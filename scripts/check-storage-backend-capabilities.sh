#!/usr/bin/env bash
# check-storage-backend-capabilities.sh — capability-truth guard for optional
# StorageBackend operations (#1087).
#
# StorageBackend carries optional methods whose default bodies return Ok(()),
# None, empty vectors, 0, or false. A new default added without thought can make
# an unsupported operation look successful (see #1087). This guard makes that
# impossible to land silently:
#
#   1. It parses the trait source and inventories every method with a default
#      body (`memory-core/src/storage/backend.rs`, or `backend/mod.rs` after the
#      slice-2 split, plus any sibling capability module).
#   2. Every defaulted method must be classified in
#      `scripts/storage-backend-capabilities.toml` — coverage is fail-closed, so
#      an unknown default is an error, not a warning.
#   3. Each declared classification is checked against the actual default body:
#      `capability-gated` needs a predicate that exists in source and (for
#      `gate_kind = error`) a body that raises Error::CapabilityUnavailable.
#   4. The machine-readable inventory in
#      `docs/generated/storage-backend-capabilities.json` must match the tree.
#      A stale artifact fails unless every difference is a forward-compatible
#      upgrade (a new `supports_*` declaration, or an already-`pending` method
#      whose body became capability-aware); those are reported as warnings so
#      the in-flight #1087 slices do not turn main red, while anything else
#      fails.
#
# Usage:
#   ./scripts/check-storage-backend-capabilities.sh --check   # default; CI gate
#   ./scripts/check-storage-backend-capabilities.sh --write   # regenerate artifact
#   ./scripts/check-storage-backend-capabilities.sh --root DIR
#
# Exit: 0 clean (warnings allowed), 1 violation or stale artifact, 2 usage error.

set -euo pipefail

if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: python3 is required" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly PROJECT_ROOT
export STORAGE_CAPABILITIES_ROOT="$PROJECT_ROOT"

exec python3 - "$@" <<'PY'
"""Capability-truth checker for the StorageBackend trait (#1087)."""

import json
import os
import re
import sys
import tomllib
from pathlib import Path

USAGE = (
    "usage: check-storage-backend-capabilities.sh "
    "[--check|--write] [--root DIR]"
)

ARTIFACT_REL = "docs/generated/storage-backend-capabilities.json"
OVERLAY_REL = "scripts/storage-backend-capabilities.toml"

# Capability predicates are recognised structurally by name and body shape, so a
# new declaration does not need a hand-written overlay entry (it is still
# inventoried in the artifact).
PREDICATE_RE = re.compile(r"^(supports_|can_)")

DEFAULTED_CLASSIFICATIONS = (
    "capability-gated",
    "value-returning-with-typed-fallback",
    "genuinely-safe-default",
)
ALL_CLASSIFICATIONS = ("required",) + DEFAULTED_CLASSIFICATIONS + (
    "capability-declaration",
)

FABRICATED_BODY_KINDS = frozenset(
    {"unit", "none", "empty-vec", "bool-false", "zero", "default", "constructor"}
)

failures: list[str] = []
warnings: list[str] = []


def fail(message: str) -> None:
    failures.append(message)


def warn(message: str) -> None:
    warnings.append(message)


def die(message: str, code: int) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    sys.exit(code)


# --------------------------------------------------------------------------
# Trait source parsing
# --------------------------------------------------------------------------


def trait_body(text: str, name: str):
    match = re.search(r"pub\s+trait\s+" + re.escape(name) + r"\b[^{]*\{", text)
    if match is None:
        return None
    start = match.end()
    depth = 1
    index = start
    while index < len(text) and depth:
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
        index += 1
    return text[start : index - 1]


def scan_methods(body: str):
    """Return [(name, kind, body_text)] in source order."""
    pattern = re.compile(r"(?m)^    ((?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\()")
    methods = []
    for match in pattern.finditer(body):
        name = match.group(2)
        cursor = body.index("(", match.start())
        parens = 0
        while cursor < len(body):
            char = body[cursor]
            if char == "(":
                parens += 1
            elif char == ")":
                parens -= 1
                if parens == 0:
                    cursor += 1
                    break
            cursor += 1
        while cursor < len(body) and body[cursor] not in "{;":
            cursor += 1
        if cursor >= len(body):
            continue
        if body[cursor] == ";":
            methods.append((name, "required", ""))
            continue
        depth = 1
        end = cursor + 1
        while end < len(body) and depth:
            if body[end] == "{":
                depth += 1
            elif body[end] == "}":
                depth -= 1
            end += 1
        methods.append((name, "defaulted", body[cursor:end]))
    return methods


def body_kind(body: str) -> str:
    flat = " ".join(body.split())
    inner = flat
    if inner.startswith("{") and inner.endswith("}"):
        inner = inner[1:-1].strip()
    if "CapabilityUnavailable" in flat or "capability_unavailable" in flat:
        return "capability-unavailable"
    if re.search(r"self\.\w+\(", flat):
        return "delegate"
    if "Ok(Vec::new())" in flat or "Ok(vec![])" in flat:
        return "empty-vec"
    if re.search(r"\bOk\(\s*None\s*\)", flat):
        return "none"
    if re.search(r"\bOk\(\s*false\s*\)", flat):
        return "bool-false"
    if re.search(r"\bOk\(\s*0\s*\)", flat):
        return "zero"
    if "::default()" in flat:
        return "default"
    if re.search(r"\bOk\(\s*[A-Za-z_][\w:]*::new\(\)\s*\)", flat):
        return "constructor"
    if re.search(r"\bOk\(\s*\(\)\s*\)", flat):
        return "unit"
    if re.fullmatch(r"(false|true)", inner):
        return "predicate-true" if inner == "true" else "predicate-false"
    return "unknown"


def is_predicate(name: str, kind: str) -> bool:
    return bool(PREDICATE_RE.match(name)) and kind.startswith("predicate-")


def discover_source(root: Path):
    primary = root / "memory-core/src/storage/backend.rs"
    if primary.is_file():
        paths = [primary]
    else:
        primary = root / "memory-core/src/storage/backend/mod.rs"
        if not primary.is_file():
            die(
                "cannot find memory-core/src/storage/backend.rs or "
                "memory-core/src/storage/backend/mod.rs under "
                f"{root}",
                1,
            )
        paths = sorted(primary.parent.glob("*.rs"))
    return paths


def load_methods(root: Path):
    methods = {}
    order = []
    source_rels = []
    for path in discover_source(root):
        text = path.read_text(encoding="utf-8")
        for trait in ("StorageBackend", "StorageBackendCapabilities"):
            body = trait_body(text, trait)
            if body is None:
                continue
            for name, kind, body_text in scan_methods(body):
                if name in methods:
                    die(f"method {name} declared in more than one trait", 1)
                methods[name] = {
                    "name": name,
                    "signature": kind,
                    "default_body": body_kind(body_text) if kind == "defaulted" else None,
                }
                order.append(name)
        source_rels.append(str(path.relative_to(root)))
    if not methods:
        die("no StorageBackend methods parsed; source layout changed?", 1)
    return methods, order, source_rels


# --------------------------------------------------------------------------
# Overlay parsing and validation
# --------------------------------------------------------------------------


def load_overlay(root: Path):
    path = root / OVERLAY_REL
    if not path.is_file():
        die(f"missing overlay {OVERLAY_REL}; cannot classify defaulted methods", 1)
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    entries = {}
    for entry in data.get("method", []):
        name = entry.get("name")
        if not name:
            die(f"overlay entry without a name in {OVERLAY_REL}", 1)
        if name in entries:
            die(f"duplicate overlay entry for {name}", 1)
        entries[name] = entry
    return entries


def validate_overlay(methods, overlay, source_text) -> None:
    for name, entry in overlay.items():
        if name not in methods:
            fail(
                f"overlay entry '{name}' has no matching StorageBackend method "
                "(stale classification; remove it)"
            )
            continue
        method = methods[name]
        classification = entry.get("classification")
        kind = method["default_body"]
        if classification not in ALL_CLASSIFICATIONS:
            fail(f"overlay entry '{name}' has invalid classification {classification!r}")
            continue
        if is_predicate(name, kind or ""):
            if classification != "capability-declaration":
                fail(
                    f"'{name}' is a capability predicate; its overlay classification must "
                    "be 'capability-declaration' (or the entry can be removed)"
                )
            continue
        if method["signature"] == "required":
            fail(
                f"overlay entry '{name}' is declared '{classification}' but the method "
                "has no default body (remove the overlay entry)"
            )
            continue
        if classification == "required":
            fail(f"overlay entry '{name}' is defaulted but declared 'required'")
            continue
        if classification in (
            "value-returning-with-typed-fallback",
            "genuinely-safe-default",
        ):
            if not str(entry.get("justification", "")).strip():
                fail(
                    f"overlay entry '{name}' ({classification}) needs a justification"
                )
        if classification == "capability-gated":
            capability = entry.get("capability")
            gate_kind = entry.get("gate_kind")
            if not capability:
                fail(f"capability-gated entry '{name}' needs a 'capability' predicate")
            elif not re.search(r"\bfn\s+" + re.escape(capability) + r"\s*\(", source_text):
                fail(
                    f"capability-gated entry '{name}' names predicate '{capability}', "
                    "which does not exist in the trait source"
                )
            if gate_kind not in ("predicate", "error"):
                fail(
                    f"capability-gated entry '{name}' needs gate_kind = "
                    "'predicate' or 'error'"
                )
            if not str(entry.get("justification", "")).strip():
                fail(f"capability-gated entry '{name}' needs a justification")


# --------------------------------------------------------------------------
# Coherence: declared classification vs actual default body
# --------------------------------------------------------------------------


def check_coherence(methods, overlay) -> None:
    for name, method in methods.items():
        if method["signature"] == "required":
            # A required method carrying an overlay entry is reported by
            # validate_overlay; here only defaulted methods need coherence checks.
            continue
        kind = method["default_body"]
        entry = overlay.get(name)
        if entry is None:
            if is_predicate(name, kind):
                if kind == "predicate-true":
                    fail(
                        f"capability predicate '{name}' defaults to true; capability "
                        "declarations must default to false"
                    )
                continue
            fail(
                f"defaulted method '{name}' has no classification in {OVERLAY_REL}; "
                "declare it (capability-gated or allowlisted with a justification) "
                "before it can land"
            )
            continue
        classification = entry.get("classification")
        if kind == "unknown":
            fail(f"defaulted method '{name}' has an unrecognised default body shape")
            continue
        if is_predicate(name, kind):
            if kind == "predicate-true":
                fail(
                    f"capability predicate '{name}' defaults to true; capability "
                    "declarations must default to false"
                )
            continue
        if classification == "capability-gated":
            gate_kind = entry.get("gate_kind")
            if gate_kind == "error" and kind != "capability-unavailable":
                fail(
                    f"'{name}' is declared capability-gated (gate_kind=error) but its "
                    f"default body is {kind}, not a CapabilityUnavailable error"
                )
            if gate_kind == "predicate" and kind == "capability-unavailable":
                warn(
                    f"'{name}' now returns CapabilityUnavailable but is declared "
                    "capability-gated (gate_kind=predicate); reclassify with --write"
                )
            if gate_kind == "predicate" and kind not in FABRICATED_BODY_KINDS:
                fail(
                    f"'{name}' is declared capability-gated (gate_kind=predicate) but "
                    f"its default body is {kind}"
                )
        elif classification == "value-returning-with-typed-fallback":
            if kind == "capability-unavailable":
                warn(
                    f"'{name}' now returns CapabilityUnavailable but is declared a typed "
                    "fallback; reclassify with --write"
                )
            elif kind not in FABRICATED_BODY_KINDS:
                fail(
                    f"'{name}' is declared a typed fallback but its default body is {kind}"
                )
        elif classification == "genuinely-safe-default":
            if kind != "delegate":
                fail(
                    f"'{name}' is declared genuinely-safe-default but its default body is "
                    f"{kind}, not a delegation"
                )


# --------------------------------------------------------------------------
# Artifact generation and staleness check
# --------------------------------------------------------------------------


def build_artifact(methods, order, source_rels, overlay):
    entries = []
    for name in order:
        method = methods[name]
        kind = method["default_body"]
        if method["signature"] == "required":
            classification = "required"
        elif is_predicate(name, kind or ""):
            classification = "capability-declaration"
        else:
            classification = overlay.get(name, {}).get("classification", "unclassified")
        entry = overlay.get(name, {})
        entries.append(
            {
                "name": name,
                "signature": method["signature"],
                "classification": classification,
                "capability": entry.get("capability"),
                "gate_kind": entry.get("gate_kind"),
                "default_body": kind,
                "justification": entry.get("justification"),
                "pending": entry.get("pending"),
            }
        )
    entries.sort(key=lambda item: item["name"])
    counts = {
        "methods_total": len(entries),
        "required": sum(1 for e in entries if e["signature"] == "required"),
        "defaulted": sum(1 for e in entries if e["signature"] == "defaulted"),
    }
    for classification in DEFAULTED_CLASSIFICATIONS + ("capability-declaration",):
        counts[classification.replace("-", "_")] = sum(
            1 for e in entries if e["classification"] == classification
        )
    return {
        "generated_by": "scripts/check-storage-backend-capabilities.sh --write",
        "trait": "StorageBackend",
        "source": source_rels,
        "summary": counts,
        "methods": entries,
    }


def compare_artifact(old, new, overlay) -> None:
    old_entries = {e["name"]: e for e in old.get("methods", [])}
    new_entries = {e["name"]: e for e in new["methods"]}

    for name in sorted(set(new_entries) - set(old_entries)):
        entry = new_entries[name]
        if entry["classification"] == "capability-declaration":
            warn(
                f"new capability declaration '{name}' is not in the checked-in "
                "inventory yet; refresh with --write"
            )
        else:
            fail(
                f"inventory is stale: new method '{name}' is not in "
                f"{ARTIFACT_REL}; run --write"
            )

    for name in sorted(set(old_entries) - set(new_entries)):
        fail(
            f"inventory is stale: method '{name}' is listed in {ARTIFACT_REL} "
            "but no longer exists; run --write"
        )

    for name in sorted(set(old_entries) & set(new_entries)):
        before, after = old_entries[name], new_entries[name]
        if before == after:
            continue
        diffs = {
            key: (before.get(key), after.get(key))
            for key in set(before) | set(after)
            if before.get(key) != after.get(key)
        }
        pending = overlay.get(name, {}).get("pending")
        upcoming = before.get("default_body") != "capability-unavailable" and after.get(
            "default_body"
        ) == "capability-unavailable"
        if pending and upcoming and set(diffs) == {"default_body"}:
            warn(
                f"'{name}' became capability-aware ({pending}); "
                "refresh the inventory with --write"
            )
        else:
            fail(
                f"inventory is stale for '{name}': {diffs}; "
                "run --write to regenerate"
            )


# --------------------------------------------------------------------------
# Entry point
# --------------------------------------------------------------------------


def report(artifact) -> int:
    for message in warnings:
        print(f"[storage-capabilities] WARN: {message}")
    for message in failures:
        print(f"[storage-capabilities] FAIL: {message}", file=sys.stderr)

    if failures:
        print(
            f"[storage-capabilities] {len(failures)} violation(s); "
            f"see {OVERLAY_REL}",
            file=sys.stderr,
        )
        return 1

    counts = artifact["summary"]
    print(
        "[storage-capabilities] OK: "
        f"{counts['defaulted']} defaulted / {counts['methods_total']} methods "
        f"classified ({counts['capability_gated']} capability-gated); "
        f"{len(warnings)} warning(s)"
    )
    return 0


def main() -> int:
    mode = "check"
    root_arg = None
    args = sys.argv[1:]
    index = 0
    while index < len(args):
        arg = args[index]
        if arg == "--check":
            mode = "check"
        elif arg in ("--write", "--generate"):
            mode = "write"
        elif arg == "--root":
            index += 1
            if index >= len(args):
                die("--root needs a directory", 2)
            root_arg = args[index]
        elif arg.startswith("--root="):
            root_arg = arg.split("=", 1)[1]
        elif arg in ("-h", "--help"):
            print(USAGE)
            return 0
        else:
            die(f"unknown argument: {arg}\n{USAGE}", 2)
        index += 1

    root = Path(
        root_arg or os.environ.get("STORAGE_CAPABILITIES_ROOT") or "."
    ).resolve()

    source_text = "\n".join(
        path.read_text(encoding="utf-8") for path in discover_source(root)
    )

    methods, order, source_rels = load_methods(root)
    overlay = load_overlay(root)

    validate_overlay(methods, overlay, source_text)
    check_coherence(methods, overlay)

    # Fail closed before touching the artifact: an unclassified or incoherent
    # tree must never regenerate or repair the inventory.
    if failures:
        return report(None)

    artifact = build_artifact(methods, order, source_rels, overlay)
    artifact_path = root / ARTIFACT_REL

    if mode == "write":
        artifact_path.parent.mkdir(parents=True, exist_ok=True)
        artifact_path.write_text(
            json.dumps(artifact, indent=2) + "\n", encoding="utf-8"
        )
        print(f"[storage-capabilities] wrote {ARTIFACT_REL}")
    else:
        if not artifact_path.is_file():
            fail(f"missing {ARTIFACT_REL}; run --write")
        else:
            try:
                old = json.loads(artifact_path.read_text(encoding="utf-8"))
            except json.JSONDecodeError as error:
                die(f"invalid JSON in {ARTIFACT_REL}: {error}", 1)
            compare_artifact(old, artifact, overlay)

    return report(artifact)


if __name__ == "__main__":
    sys.exit(main())
PY
