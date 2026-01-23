#!/bin/bash
# Check if documentation is up-to-date
# This script can be used in CI/CD or as a pre-commit hook

set -e

echo "🔍 Checking if documentation is up-to-date..."

# Build the project
cargo build > /dev/null 2>&1

# Store original docs for comparison
TEMP_TASKS="/tmp/driftless-tasks-check.md"
TEMP_FACTS="/tmp/driftless-facts-check.md"
TEMP_LOGS="/tmp/driftless-logs-check.md"
TEMP_TEMPLATE="/tmp/driftless-template-check.md"

cp docs/tasks-reference.md "$TEMP_TASKS" 2>/dev/null || true
cp docs/facts-reference.md "$TEMP_FACTS" 2>/dev/null || true
cp docs/logs-reference.md "$TEMP_LOGS" 2>/dev/null || true
cp docs/template-reference.md "$TEMP_TEMPLATE" 2>/dev/null || true

# Regenerate docs (this will update all files)
./target/debug/driftless docs --format markdown > /dev/null 2>&1

# Check if task docs are different
if ! diff -q "$TEMP_TASKS" docs/tasks-reference.md > /dev/null 2>&1; then
    echo "❌ docs/tasks-reference.md is out of date!"
    echo "Run: ./scripts/generate-docs.sh"
    exit 1
fi

# Check if facts docs are different
if ! diff -q "$TEMP_FACTS" docs/facts-reference.md > /dev/null 2>&1; then
    echo "❌ docs/facts-reference.md is out of date!"
    echo "Run: ./scripts/generate-docs.sh"
    exit 1
fi

# Check if logs docs are different
if ! diff -q "$TEMP_LOGS" docs/logs-reference.md > /dev/null 2>&1; then
    echo "❌ docs/logs-reference.md is out of date!"
    echo "Run: ./scripts/generate-docs.sh"
    exit 1
fi

# Check if template docs are different
if ! diff -q "$TEMP_TEMPLATE" docs/template-reference.md > /dev/null 2>&1; then
    echo "❌ docs/template-reference.md is out of date!"
    echo "Run: ./scripts/generate-docs.sh"
    exit 1
fi

# Clean up
rm -f "$TEMP_TASKS" "$TEMP_FACTS" "$TEMP_LOGS" "$TEMP_TEMPLATE"

echo "✅ Documentation is up-to-date"