# Podman Devcontainer Troubleshooting

This guide covers common Podman + VS Code Dev Containers issues for Driftless development on Linux and macOS.

## Scope

- Primary target: Linux and macOS developer workstations
- Windows: best-effort only (prefer WSL2 workflow)

## Baseline Checks

Verify Podman is healthy before opening the devcontainer:

```bash
podman info
podman ps
```

If VS Code Dev Containers is configured to use Podman, confirm your VS Code setting:

```json
{
  "dev.containers.dockerPath": "podman"
}
```

## Rootless vs Rootful

- Prefer **rootless** Podman for day-to-day development.
- Keep the devcontainer user non-root (`vscode`) to avoid bind-mount ownership drift.
- If you temporarily switch root mode or engine mode, rebuild and clean artifacts:

```bash
cargo clean
```

## Socket and Engine Wiring

Dev Containers expects a Docker-compatible interface. With Podman Desktop this is typically configured for you, but if it breaks:

1. Verify Podman engine works (`podman info`).
2. Verify VS Code points to Podman (`dev.containers.dockerPath`).
3. Rebuild the container from VS Code: **Dev Containers: Rebuild Container**.

## Common Problems

### Permission denied running test binaries

Symptoms:

- `Permission denied (os error 13)` when running test executables in `target/debug/deps`

Likely cause:

- Stale build artifacts after switching user/engine/root mode.

Fix:

```bash
cargo clean
./scripts/validate.sh
```

### Linker killed (`ld terminated with signal 9`)

Symptoms:

- Linking fails with signal 9 during tests or release builds.

Likely cause:

- VM/container out-of-memory condition.

Fixes:

```bash
CARGO_BUILD_JOBS=2 cargo test --all --quiet
CARGO_BUILD_JOBS=2 cargo build --release
./scripts/build-release-local.sh
```

Also increase memory assigned to Podman Desktop when possible.

Use `cargo build --release` in CI/release pipelines; `release-local` is intended for local developer machines with tighter memory limits.

### Devcontainer no longer starts cleanly

Recommended reset sequence:

1. Rebuild container from VS Code.
2. If still broken, remove and recreate the container.
3. Run `cargo clean` inside the rebuilt container.

## Team Recommendations

- Standardize on one documented Podman profile (CPU/memory and rootless mode).
- Keep `CARGO_BUILD_JOBS` conservative for laptop-class machines.
- Ask contributors to run `./scripts/validate.sh --fail-fast` before opening PRs.