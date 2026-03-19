#!/bin/bash
# scripts/update-changelog.sh
# Updates CHANGELOG.md using git-cliff

set -e

if ! command -v git-cliff &> /dev/null; then
    echo "git-cliff is not installed. Skipping changelog update."
    exit 0
fi

echo "Updating CHANGELOG.md with git-cliff..."
git-cliff --output CHANGELOG.md

echo "Changelog updated successfully."
