#!/usr/bin/env bash
set -euo pipefail

echo "[clean-artifacts] removing build and transient artifacts..."

if [ -d target ]; then
  rm -rf target
  echo "[clean-artifacts] removed target/"
else
  echo "[clean-artifacts] target/ not present"
fi

if [ -d node_modules ]; then
  rm -rf node_modules
  echo "[clean-artifacts] removed node_modules/"
else
  echo "[clean-artifacts] node_modules/ not present"
fi

if [ -d .cargo-cache ]; then
  rm -rf .cargo-cache
  echo "[clean-artifacts] removed .cargo-cache/"
fi

echo "[clean-artifacts] done"
