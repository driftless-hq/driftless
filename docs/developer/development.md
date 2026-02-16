# Development Guide

This guide covers the development workflow for contributing to Driftless.

## Prerequisites

- Rust (stable, beta, or MSRV 1.92)
- Git
- Python 3.x (for documentation generation)
- Visual Studio Code with the Dev Containers extension (recommended)
- A container engine for devcontainers:
	- Linux: Docker Engine or Podman
	- macOS: Docker Desktop or Podman Desktop
	- Windows: Docker Desktop or Podman Desktop (best effort only; see Windows notes below)

## Platform Support Policy

- **Linux:** Fully supported for development and CI parity.
- **macOS:** Fully supported for development via devcontainers (Docker Desktop or Podman Desktop).
- **Windows:** **Best effort** support.
	- We accept fixes for clear Windows-specific edge cases.
	- We do not guarantee full parity for all local workflows.
	- Windows user/runtime support for the binary is also best effort.

## Devcontainers (Recommended)

Using a devcontainer is the recommended way to get a consistent toolchain across machines.

### Linux

- **Docker:** install Docker Engine and verify `docker ps` works.
- **Podman:** install Podman and verify `podman ps` works.

For Podman with VS Code Dev Containers, configure VS Code to use Podman:

```json
{
	"dev.containers.dockerPath": "podman"
}
```

### macOS

- **Docker Desktop:** install Docker Desktop and verify `docker ps` works.
- **Podman Desktop:** install Podman Desktop, initialize/start the Podman machine, and verify `podman ps` works.

For Podman with VS Code Dev Containers on macOS:

1. Set VS Code to use Podman:

```json
{
	"dev.containers.dockerPath": "podman"
}
```

2. Ensure a Docker-compatible socket is exposed (Podman Desktop normally configures this).
3. Rebuild the devcontainer after changing engine settings.

### Windows (Best Effort)

- Preferred path is VS Code + Dev Containers with Docker Desktop or Podman Desktop.
- WSL2-based development is typically more reliable than native Windows filesystem mounts.
- Limit expectations to Windows-specific edge-case fixes, not full parity for every local setup.

### Open the Repository in a Devcontainer

1. Clone the repository.
2. Open it in VS Code.
3. Run **Dev Containers: Reopen in Container**.
4. After engine changes (Docker ↔ Podman, rootful ↔ rootless, or user changes), run a clean build:

```bash
cargo clean
```

## Setting Up Your Development Environment

If you are not using a devcontainer, use this native setup:

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

### Resource Guidance for macOS/Windows VMs

Containerized Rust builds can be memory-intensive (especially linking tests and release binaries).

- If you see linker failures like `ld terminated with signal 9 [Killed]`, the VM/container likely hit OOM.
- Increase memory available to Docker Desktop/Podman Desktop, and/or reduce Cargo parallelism:

```bash
CARGO_BUILD_JOBS=2 cargo test --all --quiet
```

- CI should continue using `cargo build --release` for the smallest, most optimized binary.
- For local builds on constrained machines, use the lower-memory profile:

```bash
cargo build --profile release-local -j 2
```

Or use the helper script:

```bash
./scripts/build-release-local.sh
```

- The validation script already respects `CARGO_BUILD_JOBS` and defaults to a conservative value.

For Podman-specific setup and recovery steps, see [Podman Devcontainer Troubleshooting](podman-troubleshooting.md).

## Running Validation Checks

Before committing your changes, you should run the validation script to catch potential CI failures early:

```bash
./scripts/validate.sh
```

This script runs all the validation checks that are performed in the CI pipeline:

1. **Code Formatting Check** - Ensures code follows Rust formatting standards (`cargo fmt --all -- --check`)
2. **Clippy Linter** - Runs the Rust linter to catch common mistakes and enforce best practices (`cargo clippy -- -D warnings`)
3. **Documentation Validation** - Verifies that generated documentation is up-to-date

By default, the script runs all checks and reports all failures. To exit immediately on the first failure, use the `--fail-fast` flag:

```bash
./scripts/validate.sh --fail-fast
```

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
