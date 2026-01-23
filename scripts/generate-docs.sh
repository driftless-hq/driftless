#!/bin/bash
# Documentation generation script for Driftless
# This script generates all documentation artifacts

set -e

echo "🔨 Building Driftless..."
cargo build

echo "🔍 Checking if docs need updating..."
if ./scripts/check-docs.sh 2>/dev/null; then
    echo "✅ Markdown docs are already up-to-date, skipping markdown generation"
else
    echo "📝 Markdown docs need updating, regenerating..."
    echo "📚 Generating documentation..."
    ./target/debug/driftless docs --format markdown --output-dir docs
fi

echo "🦀 Generating Rust API documentation..."
cargo doc --no-deps --document-private-items

echo "✅ Documentation generation complete!"
echo ""
echo "Generated files:"
echo "  - docs/tasks-reference.md (Task documentation)"
echo "  - docs/facts-reference.md (Facts documentation)"
echo "  - docs/logs-reference.md (Logs documentation)"
echo "  - docs/template-reference.md (Template documentation)"
echo "  - target/doc/ (Rust API documentation)"
echo ""
echo "To view Rust documentation locally:"
echo "  cargo doc --open"