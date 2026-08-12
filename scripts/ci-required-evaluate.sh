#!/usr/bin/env bash
# ci-required-evaluate.sh — single result-mapping authority for the CI / Required
# aggregate (ADR-079 CIT-A1, same-run fast gate). Accepts one or more quoted
# 'name=result' arguments and ONLY accepts result=success.
#
# Fails (::error:: + nonzero) for:
#   - result=failure | cancelled | timed_out | skipped
#   - empty/malformed argument (missing '=', empty name or result, extra '=')
#   - unknown result value
#   - zero arguments (missing results)
#
# Prints exactly one success line and exits 0 only when EVERY argument is success.

set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "::error::CI / Required aggregate missing dependency results (no name=result arguments provided)" >&2
  exit 1
fi

PASSED=1

for arg in "$@"; do
  # Exactly one '=' separator with non-empty name and non-empty result.
  if [[ "$arg" != *=* ]]; then
    echo "::error::CI / Required aggregate received malformed argument '$arg' (expected name=result)" >&2
    PASSED=0
    continue
  fi
  name="${arg%%=*}"
  result="${arg#*=}"
  if [[ -z "$name" || -z "$result" || "$result" == *=* ]]; then
    echo "::error::CI / Required aggregate received malformed argument '$arg' (expected exactly one '=' with non-empty name and result)" >&2
    PASSED=0
    continue
  fi
  case "$result" in
    success)
      echo "OK: $name"
      ;;
    failure|cancelled|timed_out|skipped)
      echo "::error::$name was $result - CI / Required aggregate failed" >&2
      PASSED=0
      ;;
    *)
      echo "::error::$name has unknown result '$result' - CI / Required aggregate failed" >&2
      PASSED=0
      ;;
  esac
done

if [[ "$PASSED" -eq 1 ]]; then
  echo "CI / Required aggregate passed"
  exit 0
fi

exit 1
