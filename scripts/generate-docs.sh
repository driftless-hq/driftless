#!/bin/bash
# Documentation generation script for Driftless
# This script generates all documentation artifacts

set -e

echo "🔨 Building Driftless..."
cargo build

echo "🔍 Checking if docs need updating..."
if ./scripts/check-docs.sh 2>/dev/null; then
    echo "✅ Docs are already up-to-date, skipping generation"
    exit 0
fi

echo "📝 Docs need updating, regenerating..."

echo "📚 Generating task documentation..."
./target/debug/driftless docs --format markdown --output docs/tasks-reference.md

echo "🎨 Generating template documentation..."
./target/debug/driftless docs --format markdown

echo "🦀 Generating Rust API documentation..."
cargo doc --no-deps --document-private-items

echo "✅ Documentation generation complete!"
echo ""
echo "Generated files:"
echo "  - docs/tasks-reference.md (Task documentation)"
echo "  - docs/template-reference.md (Template documentation)"
echo "  - target/doc/ (Rust API documentation)"
echo ""
echo "To view Rust documentation locally:"
echo "  cargo doc --open"