#!/bin/bash
# Validation script for Driftless
# This script runs all validation checks that are performed in the CI/CD pipeline
# Run this locally before committing to catch potential pipeline failures early

set -e

echo "🔍 Running Driftless validation checks..."
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED=0
FAIL_FAST=false
CARGO_JOBS=${CARGO_BUILD_JOBS:-2}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--fail-fast]"
            exit 1
            ;;
    esac
done

# Function to run a validation step
run_validation() {
    local name=$1
    shift
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "▶ Running: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if "$@"; then
        echo -e "${GREEN}✅ $name passed${NC}"
    else
        echo -e "${RED}❌ $name failed${NC}"
        FAILED=1
        if [ "$FAIL_FAST" = true ]; then
            exit 1
        fi
    fi
    echo ""
}

# 1. Check code formatting
run_validation "Code Formatting Check" cargo fmt --all -- --check

# 2. Run clippy linter
run_validation "Clippy Linter" cargo clippy -j "$CARGO_JOBS" -- -D warnings

# 3. Run tests
run_validation "Unit Tests" cargo test --all --quiet -j "$CARGO_JOBS"

# 4. Check documentation is up-to-date
run_validation "Documentation Validation" ./scripts/check-docs.sh

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All validation checks passed!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo -e "${RED}❌ Some validation checks failed${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "To fix formatting issues, run:"
    echo "  cargo fmt --all"
    echo ""
    echo "To fix documentation, run:"
    echo "  ./scripts/generate-docs.sh"
    echo ""
    echo "For clippy warnings, fix the issues manually or check the output above."
    echo ""
    echo "Tip: Use --fail-fast to exit on the first failure"
    exit 1
fi
