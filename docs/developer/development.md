# Development Guide

This guide covers the development workflow for contributing to Driftless.

## Prerequisites

- Rust (stable, beta, or MSRV 1.92)
- Git
- Python 3.x (for documentation generation)

## Setting Up Your Development Environment

1. Clone the repository:
```bash
git clone https://github.com/driftless-hq/driftless.git
cd driftless
```

2. Build the project:
```bash
cargo build
```

3. Run tests:
```bash
cargo test
```

## Running Validation Checks

Before committing your changes, you should run the validation script to catch potential CI failures early:

```bash
./scripts/validate.sh
```

This script runs all the validation checks that are performed in the CI pipeline:

1. **Code Formatting Check** - Ensures code follows Rust formatting standards (`cargo fmt --all -- --check`)
2. **Clippy Linter** - Runs the Rust linter to catch common mistakes and enforce best practices (`cargo clippy -- -D warnings`)
3. **Documentation Validation** - Verifies that generated documentation is up-to-date

### Fixing Validation Issues

If validation checks fail, here's how to fix them:

#### Formatting Issues

Run cargo fmt to automatically fix formatting:
```bash
cargo fmt --all
```

#### Clippy Warnings

Review the clippy output and fix the issues manually. The warnings will guide you on what needs to be changed.

#### Documentation Issues

Regenerate the documentation:
```bash
./scripts/generate-docs.sh
```

## Building Documentation

To generate and view documentation locally:

```bash
# Generate all documentation
./scripts/generate-docs.sh

# View Rust API documentation in your browser
cargo doc --open
```

## Running the Project Locally

```bash
# Run in development mode
cargo run -- --help

# Run with specific command
cargo run -- apply --dry-run
```

## Submitting Changes

1. Run validation checks: `./scripts/validate.sh`
2. Commit your changes
3. Push to your fork
4. Create a pull request

The CI pipeline will automatically run the same validation checks on your pull request.
