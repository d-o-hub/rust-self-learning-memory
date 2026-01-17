#!/bin/bash
# Check for unwrap() calls in production code (excluding tests)
# Exit with error if found

set -e

echo "🔍 Checking for unwrap() in production code..."

# Find all Rust files, exclude test files
production_files=$(find memory-core/src memory-storage-*/src memory-mcp/src memory-cli/src \
  -name "*.rs" -type f \
  ! -name "*test*.rs" \
  ! -name "tests.rs")

unwrap_count=0
files_with_unwrap=""

for file in $production_files; do
  # Get line number where #[cfg(test)] starts
  test_line=$(grep -n "^#\[cfg(test)\]" "$file" 2>/dev/null | head -1 | cut -d: -f1 || echo "")
  
  if [ -n "$test_line" ]; then
    # Has test module, check only production code (before test section)
    prod_unwrap=$(head -n $((test_line - 1)) "$file" 2>/dev/null | \
      grep "\.unwrap()" | \
      grep -v "^\s*//" | \
      grep -v "unwrap_or" || true)
  else
    # No test module, check entire file
    prod_unwrap=$(grep "\.unwrap()" "$file" 2>/dev/null | \
      grep -v "^\s*//" | \
      grep -v "unwrap_or" | \
      grep -v "///" || true)
  fi
  
  if [ -n "$prod_unwrap" ]; then
    unwrap_count=$((unwrap_count + 1))
    files_with_unwrap="$files_with_unwrap\n  - $file"
    echo "⚠️  Found unwrap() in: $file"
    echo "$prod_unwrap" | head -3
    echo ""
  fi
done

if [ $unwrap_count -gt 0 ]; then
  echo ""
  echo "❌ Found unwrap() in $unwrap_count production file(s):"
  echo -e "$files_with_unwrap"
  echo ""
  echo "Please use proper error handling instead:"
  echo "  - Use '?' operator for error propagation"
  echo "  - Use 'unwrap_or()' or 'unwrap_or_else()' for safe defaults"
  echo "  - Use 'ok_or()' to convert Option to Result"
  echo ""
  echo "Test code may use unwrap() freely."
  exit 1
else
  echo "✅ No unwrap() found in production code!"
  exit 0
fi
