#!/bin/bash
# Local release build helper for constrained developer environments

set -e

CARGO_JOBS=${CARGO_BUILD_JOBS:-2}

echo "🚀 Building Driftless with release-local profile..."
echo "   Profile: release-local"
echo "   Jobs: ${CARGO_JOBS}"
echo ""

cargo build --profile release-local -j "$CARGO_JOBS"

echo ""
echo "✅ Local release build completed"