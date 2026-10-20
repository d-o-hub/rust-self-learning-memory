#!/usr/bin/env bash
# validate-schema-sql.sh — ACT-406: hand-written SQL columns vs the Turso DDL.
#
# The v0.1.45 nightly was red for days because the monitoring SQL named columns
# the DDL does not create (and wrote a seconds value into an `_ms` column). No
# local gate could see that: the schema is authoritative in the `CREATE TABLE`
# constants, while the writes live in `INSERT`/`UPDATE` string literals, and
# nothing linked them. This sensor links the two.
#
# Usage:
#   ./scripts/validate-schema-sql.sh              # validate the Turso crate
#   ./scripts/validate-schema-sql.sh --self-test   # prove the check is load-bearing
#
# Fails (exit 1) when a hand-written INSERT references a column that the table's
# CREATE TABLE does not define. A column defined in the DDL but absent from an
# INSERT is fine (it relies on a DEFAULT) and is never flagged.
#
# Scope: v1 covers INSERT column lists against `CREATE TABLE` (not
# `CREATE VIRTUAL TABLE`/FTS) across non-test sources. UPDATE-SET and FTS
# coverage are documented follow-ups on issue #1199.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TURSO_SRC="$PROJECT_ROOT/memory-storage-turso/src"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

fail() {
  echo -e "${RED}FAIL${NC}: $*" >&2
  exit 1
}

[[ -d "$TURSO_SRC" ]] || fail "missing Turso source dir: $TURSO_SRC"

# Emit the list of non-test Rust sources to scan. Test fixtures (files named
# *_tests.rs / tests.rs, or under a tests/ dir) own throwaway schemas and are
# excluded so they cannot mint false positives.
scan_files() {
  local root="$1"
  find "$root" -type f -name '*.rs' \
    ! -name '*_test.rs' ! -name '*_tests.rs' ! -name 'tests.rs' \
    ! -path '*/tests/*' | sort
}

# awk program: first collect every CREATE TABLE column, then validate every
# INSERT column list. Keywords are matched in their canonical uppercase form
# used throughout this crate (CREATE TABLE / INSERT ... INTO / VALUES).
# shellcheck disable=SC2016  # the awk program is a single-quoted literal; $0 etc. must not expand
DDL_AND_SQL_AWK='
BEGIN { intbl = ""; inbuf = "" }
# ---- CREATE TABLE <name> ( : start a column block ----
/CREATE TABLE[ \t]/ {
  s = $0
  sub(/.*CREATE TABLE[ \t]+/, "", s)
  sub(/^IF[ \t]+NOT[ \t]+EXISTS[ \t]+/, "", s)
  tbl = s; gsub(/[^A-Za-z0-9_].*/, "", tbl)
  if (tbl != "") { cur_ddl = tbl; inside_ddl = 1 }
  next
}
inside_ddl {
  l = $0
  gsub(/^[ \t]+/, "", l)
  if (l ~ /^\)/) { inside_ddl = 0; next }
  if (l == "") next
  n = split(l, tok, /[ \t(]+/)
  colname = tok[1]
  if (colname !~ /^[A-Za-z_][A-Za-z0-9_]*$/) next
  # A column line is "name TYPE ...". Table-level constraints (PRIMARY KEY (...),
  # CHECK (...), FOREIGN KEY, CONSTRAINT ...) are excluded because their second
  # token is not a type. Keying on the type — not on skipping keywords — keeps
  # columns that happen to be named key/value/check.
  t = toupper(tok[2])
  gsub(/[^A-Z0-9_]/, "", t)   # "INTEGER," -> "INTEGER"; keeps F32_BLOB intact
  if (t == "INTEGER" || t == "TEXT" || t == "REAL" || t == "BLOB" ||
      t == "NUMERIC" || t == "BOOLEAN" || t == "VARCHAR" || t == "CHAR" ||
      t == "BIGINT" || t == "SMALLINT" || t == "DOUBLE" || t == "FLOAT" ||
      t == "TIMESTAMP" || t == "DATETIME" || t == "DATE" || t == "DECIMAL" ||
      t == "F32_BLOB") {
    cols[cur_ddl SUBSEP colname] = 1
  }
  next
}
# ---- INSERT ... INTO <name> ( cols ) ----
# Anchor on the "(" that immediately follows the table name (match() finds the
# leftmost such span), then read up to the first ")" as the column list. This
# handles both single-line and wrapped lists without a greedy match swallowing
# the VALUES clause.
intbl == "" && match($0, /INTO[ \t]+[A-Za-z_]+[ \t]*\(/) {
  intbl = substr($0, RSTART, RLENGTH)
  sub(/.*INTO[ \t]+/, "", intbl)
  gsub(/[^A-Za-z0-9_].*/, "", intbl)
  rest = substr($0, RSTART + RLENGTH)
  p = index(rest, ")")
  if (p > 0) {
    check_list(intbl, substr(rest, 1, p - 1))
    intbl = ""
  } else {
    inbuf = rest
  }
  next
}
intbl != "" {
  l = $0
  p = index(l, ")")
  if (p > 0) {
    check_list(intbl, inbuf substr(l, 1, p - 1))
    intbl = ""
    inbuf = ""
  } else {
    inbuf = inbuf " " l
  }
  next
}
function check_list(tbl, list,   n, parts, i, col) {
  if (!(tbl in seen_tbl)) {
    # Only tables with DDL in this crate are validated; skip unknowns so FTS or
    # foreign tables neither pass nor fail here.
    if (tbl_has_ddl(tbl)) { seen_tbl[tbl] = 1 } else { return }
  }
  gsub(/\n/, " ", list)
  n = split(list, parts, ",")
  for (i = 1; i <= n; i++) {
    col = parts[i]
    gsub(/^[ \t]+/, "", col)          # trim, THEN take the leading identifier
    gsub(/[ \t]+$/, "", col)
    gsub(/[^A-Za-z0-9_].*/, "", col)  # a column token is a bare word
    if (col == "") continue
    if (!((tbl SUBSEP col) in cols)) {
      printf "%s:%d: %s references column `%s` absent from its CREATE TABLE\n",
        FILENAME, FNR, tbl, col
      bad++
    }
  }
}
function tbl_has_ddl(tbl,   k) {
  for (k in cols) { if (index(k, tbl SUBSEP) == 1) return 1 }
  return 0
}
END { if (bad) exit 1 }
'

run_validate() {
  local files
  files="$(scan_files "$TURSO_SRC")"
  # shellcheck disable=SC2086
  output="$(awk "$DDL_AND_SQL_AWK" $files 2>&1 || true)"
  if [[ -n "$output" ]]; then
    echo -e "${RED}Schema/SQL column mismatch detected:${NC}" >&2
    echo "$output" >&2
    return 1
  fi
  return 0
}

# --self-test: run the same awk over a scratch tree that plants a bad column and
# a valid column, asserting the bad one is caught and the valid one is not.
run_self_test() {
  local tmp
  tmp="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$tmp'" EXIT
  cat >"$tmp/case.rs" <<'RUST'
pub const CREATE_THING_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS thing (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    duration_ms INTEGER NOT NULL DEFAULT 0
)
"#;
const GOOD: &str = "INSERT OR REPLACE INTO thing (id, name, duration_ms) VALUES (?, ?, ?)";
const BAD: &str = "INSERT INTO thing (id, name, missing_column) VALUES (?, ?, ?)";
RUST
  awk "$DDL_AND_SQL_AWK" "$tmp/case.rs" >"$tmp/out" 2>&1 || true
  if grep -q "missing_column" "$tmp/out" && ! grep -q "duration_ms absent" "$tmp/out"; then
    echo -e "${GREEN}self-test OK${NC}: planted violation caught, valid column ignored."
    return 0
  fi
  echo -e "${RED}self-test FAILED${NC}: expected a violation for \`missing_column\`. Output:" >&2
  cat "$tmp/out" >&2
  return 1
}

if [[ "${1:-}" == "--self-test" ]]; then
  run_self_test
  exit $?
fi

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  sed -n '2,24p' "${BASH_SOURCE[0]}"
  exit 0
fi

if run_validate; then
  echo -e "${GREEN}validate-schema-sql OK${NC}: every hand-written INSERT column exists in the Turso DDL."
else
  fail "a SQL statement references a column the schema does not create (see list above)"
fi
