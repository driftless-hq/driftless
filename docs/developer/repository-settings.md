# Repository Settings Management

This document describes how repository settings are managed programmatically using GitHub Actions.

## Overview

The `driftless-hq/driftless` repository uses an automated workflow to enforce repository settings consistently. Settings are defined in `.github/repo-settings.yml` and applied automatically when changes are made to the `.github` directory.

## Configuration File

All repository settings are defined in `.github/repo-settings.yml`. This file includes:

### Repository Settings
- **Basic Information**: Description, homepage URL, topics
- **Features**: Issues, Wiki, Projects, Downloads
- **Merge Settings**: Squash merge, merge commits, rebase merging, auto-merge
- **Branch Management**: Auto-delete branches after merge

### Branch Protection

Branch protection rules for the `main` branch include:

- **Pull Request Reviews**
  - Minimum number of required approvals (default: 1)
  - Dismiss stale reviews on new commits
  - Code owner review requirements

- **Status Checks**
  - Required checks that must pass before merging:
    - Test (ubuntu-latest, amd64, stable)
    - Test (ubuntu-latest, amd64, beta)
    - Test (ubuntu-latest, amd64, 1.92)
    - Security Audit
    - Unused Dependencies
    - Outdated Dependencies
    - Build Documentation
  - Require branches to be up to date before merging

- **Additional Protections**
  - Require conversation resolution before merging
  - Prevent force pushes
  - Prevent deletions
  - Optional: Require linear history
  - Optional: Require signed commits

### GitHub Pages

- **Build Type**: GitHub Actions (not branch-based)
- **Source**: Automatically deployed from workflow

### Security

- **Vulnerability Alerts**: Enabled
- **Automated Security Fixes**: Enabled (Dependabot)

## Enforcement Workflow

The `.github/workflows/enforce-repo-settings.yml` workflow automatically applies settings when:

1. Changes are pushed to the `main` branch that modify files in `.github/`
2. The workflow is manually triggered via `workflow_dispatch`

### Workflow Steps

1. **Checkout**: Retrieves the repository code
2. **Read Settings**: Validates that `.github/repo-settings.yml` exists
3. **Install Tools**: Installs `yq` for YAML parsing
4. **Apply Settings**: Uses GitHub API to update:
   - Repository metadata and features
   - Repository topics
   - Branch protection rules
   - Security settings
5. **Verify**: Confirms settings were applied correctly

## Making Changes

To modify repository settings:

1. Edit `.github/repo-settings.yml` with your desired changes
2. Create a pull request
3. After the PR is merged to `main`, the workflow will automatically apply the new settings

### Example: Change Required Approvals

```yaml
branch_protection:
  main:
    required_pull_request_reviews:
      required_approving_review_count: 2  # Changed from 1 to 2
```

### Example: Add a New Required Status Check

```yaml
branch_protection:
  main:
    required_status_checks:
      contexts:
        - "Test (ubuntu-latest, amd64, stable)"
        - "My New Check"  # Add your new check here
```

## Permissions

The workflow uses the default `GITHUB_TOKEN` which has limited permissions. Some settings may require:

- Repository admin access
- A Personal Access Token (PAT) with `repo` and `admin:repo_hook` scopes

If the workflow fails with permission errors, consider:

1. Using a PAT stored as a repository secret
2. Granting additional permissions to the default token (if supported by GitHub)
3. Applying sensitive settings manually through the GitHub UI

## Troubleshooting

### Workflow Fails with Permission Errors

**Issue**: The workflow cannot apply certain settings due to insufficient permissions.

**Solution**:
- Some settings require repository admin access
- The default `GITHUB_TOKEN` may not have sufficient permissions
- Consider using a PAT or applying settings manually

### Settings Not Applied

**Issue**: Changes to `.github/repo-settings.yml` don't trigger the workflow.

**Solution**:
- Ensure changes are merged to the `main` branch
- Check that the workflow file exists at `.github/workflows/enforce-repo-settings.yml`
- Manually trigger the workflow using the "Actions" tab in GitHub

### Status Checks Not Found

**Issue**: Branch protection complains that status checks don't exist.

**Solution**:
- Status checks must run at least once before they can be required
- Create a test PR to trigger CI workflows
- After workflows run, the checks will be available

## Manual Settings Application

To manually apply settings without pushing to `main`:

1. Go to the repository's "Actions" tab
2. Select "Enforce Repository Settings" workflow
3. Click "Run workflow"
4. Select the `main` branch
5. Click "Run workflow" button

## Related Documentation

- [Branch Protection Setup Guide](../../.github/BRANCH_PROTECTION.md)
- [GitHub Docs: Managing Branch Protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches)
- [GitHub Docs: Repository Settings API](https://docs.github.com/en/rest/repos/repos)
