# Driftless

[![Build Status](https://github.com/driftless-hq/driftless/workflows/CI/badge.svg)](https://github.com/driftless-hq/driftless/actions)
[![codecov](https://codecov.io/gh/driftless-hq/driftless/branch/main/graph/badge.svg)](https://codecov.io/gh/driftless-hq/driftless)
[![Documentation](https://img.shields.io/badge/docs-generated-blue)](https://driftless-hq.github.io/driftless/)

> **Warning:** This is an experimental, AI-assisted project. Bugs and other shortcomings are expected and should be reported as GitHub Issues. PRs are welcome!

A lightweight Rust agent for declarative system configuration, metrics gathering, and log forwarding via GitOps.

## Documentation

- **[Complete Documentation](https://driftless-hq.github.io/driftless/)** - Full documentation on GitHub Pages
- **[Documentation Source](docs/)** - Markdown documentation in the repository
- **[API Documentation](https://driftless-hq.github.io/driftless/api/driftless/)** - Rust API documentation

## Quick Start

```bash
# Install
cargo install driftless

# Create configuration
mkdir -p ~/.config/driftless/config

# Run
driftless apply
```

For detailed installation instructions, configuration examples, and comprehensive guides, see the [full documentation](https://driftless-hq.github.io/driftless/).

## Contributing

Before submitting a pull request, please run the validation script to catch potential CI failures:

```bash
./scripts/validate.sh
```

This will check code formatting, run linting, and validate documentation. See the [Development Guide](docs/developer/development.md) for more information.

## License

Licensed under the Apache License, Version 2.0.
