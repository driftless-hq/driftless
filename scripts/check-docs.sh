#!/bin/bash
# Check if documentation is up-to-date
# This script can be used in CI/CD or as a pre-commit hook

set -e

echo "🔍 Checking if documentation is up-to-date..."

# Build the project
cargo build > /dev/null 2>&1

# Generate current docs to temp location
TEMP_DOCS="/tmp/driftless-docs-check.md"

./target/debug/driftless docs --format markdown --output "$TEMP_DOCS"

# Check if docs are different
if ! diff -q "$TEMP_DOCS" docs/tasks-reference.md > /dev/null 2>&1; then
    echo "❌ docs/tasks-reference.md is out of date!"
    echo "Run: ./scripts/generate-docs.sh"
    exit 1
fi

# Clean up
rm -f "$TEMP_DOCS"

echo "✅ Documentation is up-to-date"