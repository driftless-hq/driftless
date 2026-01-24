# Release Workflow

This document explains the release process for Driftless, including how to trigger releases manually or via version tags.

## Table of Contents

- [Overview](#overview)
- [Rationale](#rationale)
- [Triggering a Release](#triggering-a-release)
  - [Manual Release via workflow_dispatch](#manual-release-via-workflow_dispatch)
  - [Automatic Release via Version Tags](#automatic-release-via-version-tags)
- [Release Process Details](#release-process-details)
- [Platform Support](#platform-support)
- [Troubleshooting](#troubleshooting)

## Overview

Driftless uses a separate release workflow that is independent from the main CI pipeline. Releases can be triggered in two ways:

1. **Manually** using GitHub Actions `workflow_dispatch` (with customizable options)
2. **Automatically** by pushing a version tag matching the pattern `vX.Y.Z`

## Rationale

The release process is separated from the standard CI workflow for several important reasons:

### 1. **Performance and Efficiency**
   - Release builds are time-consuming, especially when building binaries for multiple platforms
   - Standard CI/CD checks (linting, testing, formatting) run frequently on every push and PR
   - Separating releases keeps the main CI pipeline fast and responsive
   - Developers get quick feedback on code quality without waiting for unnecessary release builds

### 2. **Resource Optimization**
   - Release builds consume significant compute resources (cross-compilation, multi-platform builds)
   - Not every merge to `main` requires a release
   - Running release builds only when needed reduces GitHub Actions usage and costs

### 3. **Clear Separation of Concerns**
   - CI pipeline focuses on **validation**: ensuring code quality, passing tests, and meeting standards
   - Release pipeline focuses on **distribution**: building artifacts and creating GitHub releases
   - This separation makes workflows easier to understand, maintain, and debug

### 4. **Flexibility and Control**
   - Manual releases allow for intentional version bumps and controlled release schedules
   - Tag-based releases enable GitOps-style workflows
   - Draft and pre-release options provide staging mechanisms

## Triggering a Release

### Manual Release via workflow_dispatch

Manual releases provide the most control and are ideal for planned releases.

#### Step-by-Step Process

1. **Navigate to GitHub Actions**
   - Go to your repository on GitHub
   - Click on the "Actions" tab
   - Select "Release" from the workflows list on the left

2. **Trigger the Workflow**
   - Click "Run workflow" button
   - Select the branch (typically `main`)

3. **Configure Release Options**

   The workflow provides several input options:

   | Input | Description | Required | Default |
   |-------|-------------|----------|---------|
   | `version` | Specific version to release (e.g., `0.2.0`). Leave empty to auto-bump. | No | - |
   | `bump` | Version bump type if no version specified (`patch`, `minor`, `major`) | No | `patch` |
   | `create_tag` | Whether to create and push a git tag | No | `true` |
   | `prerelease` | Mark the release as a pre-release | No | `false` |
   | `draft` | Create the release as a draft | No | `false` |

   #### Example Scenarios

   **Scenario 1: Patch Release (auto-bump)**
   - Leave `version` empty
   - Set `bump` to `patch`
   - Leave other options at defaults
   - This will bump from `0.1.0` → `0.1.1`

   **Scenario 2: Specific Version**
   - Set `version` to `0.2.0`
   - Leave other options at defaults
   - This will release exactly version `0.2.0`

   **Scenario 3: Pre-release**
   - Set `version` to `0.2.0-beta.1`
   - Set `prerelease` to `true`
   - Set `draft` to `false`
   - This creates a pre-release that's visible but marked as unstable

   **Scenario 4: Draft Release**
   - Set `version` to `1.0.0`
   - Set `draft` to `true`
   - This creates a draft release that you can review and publish later

4. **Monitor Progress**
   - The workflow will appear in the Actions tab
   - Click on the running workflow to see real-time logs
   - The workflow consists of:
     - **Build jobs**: Compile binaries for each supported platform
     - **Release job**: Create GitHub release with artifacts

5. **Verify the Release**
   - Once complete, navigate to the "Releases" page
   - Verify the release version, notes, and artifacts

### Automatic Release via Version Tags

Tag-based releases enable a GitOps workflow where pushing a version tag automatically triggers a release.

#### Tag Format

Tags must follow semantic versioning with a `v` prefix:

- **Standard releases**: `vX.Y.Z` (e.g., `v0.1.0`, `v1.2.3`)
- **Pre-releases**: `vX.Y.Z-<label>` (e.g., `v0.2.0-beta.1`, `v1.0.0-rc.2`)

Where:
- `X` = Major version
- `Y` = Minor version
- `Z` = Patch version
- `<label>` = Optional pre-release identifier

#### Step-by-Step Process

1. **Ensure Your Local Repository is Up-to-Date**
   ```bash
   git checkout main
   git pull origin main
   ```

2. **Update the Version in Cargo.toml** (if needed)
   ```bash
   # Edit Cargo.toml manually or use cargo-release
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
   
   # Commit the change
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

3. **Create and Push the Tag**
   ```bash
   # Create an annotated tag
   git tag -a v0.2.0 -m "Release v0.2.0"
   
   # Push the tag to GitHub
   git push origin v0.2.0
   ```

4. **Automatic Release**
   - Pushing the tag automatically triggers the release workflow
   - GitHub Actions will build binaries and create a release
   - The release will be published automatically (not a draft)

#### Alternative: Using cargo-release

For a more integrated approach, you can use `cargo-release`:

```bash
# Install cargo-release (if not already installed)
cargo install cargo-release

# Perform a patch release (0.1.0 → 0.1.1)
cargo release patch --execute

# Perform a minor release (0.1.0 → 0.2.0)
cargo release minor --execute

# Perform a major release (0.1.0 → 1.0.0)
cargo release major --execute

# Create a specific version
cargo release --execute 0.3.0
```

`cargo-release` will:
- Update the version in `Cargo.toml`
- Create a git commit
- Create and push a git tag
- This tag push will trigger the release workflow

## Release Process Details

### What Happens During a Release

1. **Build Phase** (runs in parallel)
   - For each supported platform:
     - Check out the code
     - Set up the Rust toolchain for the target platform
     - Build the release binary with optimizations
     - Upload the binary as an artifact

2. **Release Phase** (runs after all builds complete)
   - Download all build artifacts
   - Determine the release version
   - Generate release notes from git history
   - Create a GitHub release with:
     - Version tag
     - Release notes
     - Binary artifacts for download
     - Optional draft/pre-release flags

### Generated Artifacts

For each supported platform, a binary artifact is created:

- `driftless-linux-amd64` - Linux x86_64 (currently implemented)
- `driftless-linux-arm64` - Linux ARM64 (planned)
- `driftless-macos-amd64` - macOS Intel (planned)
- `driftless-macos-arm64` - macOS Apple Silicon (planned)
- `driftless-windows-amd64.exe` - Windows x86_64 (planned)
- `driftless-windows-arm64.exe` - Windows ARM64 (planned)

### Release Notes

Release notes are automatically generated and include:

- Version number
- List of commits since the previous release
- Installation instructions for each platform
- Links to the binary artifacts

## Platform Support

### Currently Implemented
- ✅ **Linux amd64** (x86_64-unknown-linux-gnu)

### Planned (Ready to Enable with Additional Setup)

The workflow includes matrix entries for these platforms, but they require additional setup:

- ⏳ **Linux arm64** (aarch64-unknown-linux-gnu) - Requires self-hosted runner or GitHub-hosted ubuntu-24.04-arm runner when available
- ⏳ **macOS amd64** (x86_64-apple-darwin) - Available on `macos-13` GitHub-hosted runner
- ⏳ **macOS arm64** (aarch64-apple-darwin) - Available on `macos-latest` GitHub-hosted runner
- ⏳ **Windows amd64** (x86_64-pc-windows-msvc) - Available on `windows-latest` GitHub-hosted runner
- ⏳ **Windows arm64** (aarch64-pc-windows-msvc) - Requires self-hosted runner or GitHub-hosted windows-11-arm runner when available

**Note**: Some platform combinations (ubuntu-24.04-arm, windows-11-arm) may require self-hosted runners or may not be available yet on GitHub Actions. To enable a platform, verify the runner is available and update the `skip_build` flag from `true` to `false` in the release workflow's build matrix.

## Troubleshooting

### Release Workflow Fails

**Problem**: The release workflow fails during the build phase.

**Solutions**:
1. Check the GitHub Actions logs for specific error messages
2. Ensure all tests pass in the CI workflow before triggering a release
3. Verify that the version number is valid semantic versioning
4. Check that there are no uncommitted changes in the repository

### Tag Already Exists

**Problem**: Cannot push a tag because it already exists.

**Solutions**:
1. List existing tags: `git tag -l`
2. Delete the local tag: `git tag -d vX.Y.Z`
3. Delete the remote tag (if needed): `git push origin :refs/tags/vX.Y.Z`
4. Create a new tag with a different version

### Version Mismatch

**Problem**: The version in `Cargo.toml` doesn't match the git tag.

**Solutions**:
1. Ensure you've updated `Cargo.toml` before creating the tag
2. Or use `cargo-release` which handles this automatically
3. If using manual workflow dispatch, specify the version explicitly

### Binary Not Building

**Problem**: A specific platform's binary fails to build.

**Solutions**:
1. Check if the platform is currently implemented (see Platform Support section)
2. Ensure the `skip_build` flag is set correctly in the workflow
3. Verify that the build matrix includes the correct target triple
4. Check platform-specific build logs for compilation errors

### Release Not Appearing

**Problem**: Tag was pushed but no release was created.

**Solutions**:
1. Verify the tag matches the pattern `vX.Y.Z` (with `v` prefix)
2. Check the Actions tab to see if the workflow ran
3. Look for errors in the workflow logs
4. Ensure GitHub Actions has write permissions for releases

## Best Practices

1. **Always test on main first**: Ensure the main branch CI passes before creating a release
2. **Use semantic versioning**: Follow the `MAJOR.MINOR.PATCH` convention
3. **Write meaningful release notes**: While auto-generated, consider editing them for clarity
4. **Use pre-releases for testing**: Mark unstable versions as pre-releases
5. **Create draft releases for major versions**: Review major releases before publishing
6. **Tag from main branch**: Always create release tags from the main branch
7. **Keep version in sync**: Ensure `Cargo.toml` version matches the tag

## Examples

### Example 1: Regular Patch Release

```bash
# Update version
sed -i 's/version = "0.1.0"/version = "0.1.1"/' Cargo.toml
git add Cargo.toml
git commit -m "chore: bump version to 0.1.1"
git push origin main

# Create and push tag
git tag -a v0.1.1 -m "Release v0.1.1: Bug fixes and improvements"
git push origin v0.1.1
```

### Example 2: Major Release via Workflow Dispatch

1. Go to Actions → Release → Run workflow
2. Set `version` to `1.0.0`
3. Set `draft` to `true` (to review before publishing)
4. Click "Run workflow"
5. Review the draft release, edit notes if needed
6. Publish the release

### Example 3: Beta Release

```bash
# Update version
sed -i 's/version = "0.2.0"/version = "0.3.0-beta.1"/' Cargo.toml
git add Cargo.toml
git commit -m "chore: bump version to 0.3.0-beta.1"
git push origin main

# Create and push tag
git tag -a v0.3.0-beta.1 -m "Release v0.3.0-beta.1: Beta testing"
git push origin v0.3.0-beta.1
```

Or via workflow dispatch:
1. Go to Actions → Release → Run workflow
2. Set `version` to `0.3.0-beta.1`
3. Set `prerelease` to `true`
4. Click "Run workflow"

## Additional Resources

- [Semantic Versioning](https://semver.org/)
- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [GitHub Actions workflow_dispatch](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#workflow_dispatch)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
