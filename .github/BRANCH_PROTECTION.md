# Branch Protection Setup Guide

This document describes how to configure branch protection rules for the main branch to ensure all CI checks pass before merging pull requests.

## Problem

PR #2 was merged to the main branch while workflow checks were still pending, resulting in failures on the main branch. This happened because branch protection rules were not configured to require status checks to pass.

## Solution

Configure branch protection rules for the `main` branch to require all CI workflow checks to pass before allowing merges.

## Setup Instructions

### Step 1: Navigate to Branch Protection Settings

1. Go to the repository on GitHub: https://github.com/driftless-hq/driftless
2. Click on **Settings** (top menu)
3. In the left sidebar, click on **Branches**
4. Under "Branch protection rules", click **Add rule** (or edit the existing rule for `main`)

### Step 2: Configure the Rule

#### Basic Settings
- **Branch name pattern**: `main`

#### Protection Settings

Enable the following settings:

1. **Require a pull request before merging**
   - ✅ Check this box
   - **Required approvals**: Set to `1` (or more if desired)
   - ✅ Dismiss stale pull request approvals when new commits are pushed

2. **Require status checks to pass before merging**
   - ✅ Check this box
   - ✅ Require branches to be up to date before merging
   
   **Required status checks** (add all of these):
   - `Test (ubuntu-latest, amd64, stable)`
   - `Test (ubuntu-latest, amd64, beta)`
   - `Test (ubuntu-latest, amd64, 1.92)`
   - `Security Audit`
   - `Unused Dependencies`
   - `Outdated Dependencies`
   - `Build Documentation`
   
   Note: The `Coverage` job only runs on push to main (not on PRs). The `Deploy to GitHub Pages` job only runs on push to main (not on PRs).

3. **Require conversation resolution before merging**
   - ✅ Check this box (optional but recommended)

4. **Do not allow bypassing the above settings**
   - ✅ Check this box (recommended for strict enforcement)

#### Optional but Recommended Settings

- **Require linear history**: Prevents merge commits (optional)
- **Include administrators**: Applies rules to repository administrators too (recommended)

### Step 3: Save Changes

Click **Create** (or **Save changes** if editing an existing rule)

## Verification

After configuring branch protection:

1. Create a new pull request
2. Verify that the "Merge pull request" button is disabled until all required checks pass
3. Push a commit that would fail CI (e.g., with formatting issues)
4. Confirm that the PR cannot be merged while checks are failing
5. Fix the issues and verify that the PR can be merged once checks pass

## Required Status Checks Reference

The following CI jobs MUST pass for every PR:

| Job Name | Description | When It Runs |
|----------|-------------|--------------|
| Test (ubuntu-latest, amd64, stable) | Tests on Linux with stable Rust | Always on PR |
| Test (ubuntu-latest, amd64, beta) | Tests on Linux with beta Rust | Always on PR |
| Test (ubuntu-latest, amd64, 1.92) | Tests on Linux with MSRV (Rust 1.92) | Always on PR |
| Security Audit | Checks for security vulnerabilities in dependencies (advisory) | Always on PR |
| Unused Dependencies | Detects unused dependencies | Always on PR |
| Outdated Dependencies | Checks for outdated dependencies (advisory) | Always on PR |
| Build Documentation | Validates documentation builds successfully | Always on PR |

The following jobs run conditionally and should NOT be added as required checks:
- `Coverage`: Only runs on push to main
- `Deploy to GitHub Pages`: Only runs on push to main
- `Release`: Only runs on version tags or manual trigger

## Troubleshooting

### Issue: Cannot find status checks to add

**Cause**: Status checks only appear in the list after they've run at least once.

**Solution**: 
1. Create a test PR
2. Let the CI workflow run
3. Return to branch protection settings - the checks should now be available

### Issue: PRs are blocked even though checks passed

**Cause**: "Require branches to be up to date" is enabled and the base branch has moved forward.

**Solution**: Update the PR branch by either:
- Rebasing on main: `git rebase main`
- Merging main: `git merge main`
- Using GitHub's "Update branch" button

## Additional Resources

- [GitHub Docs: About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub Docs: Managing a branch protection rule](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule)
- [GitHub Docs: About required status checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks#types-of-status-checks-on-github)

## GitHub Pages Setup

For the `Deploy to GitHub Pages` job to work, you need to configure GitHub Pages in the repository settings:

### Initial Setup

1. Go to **Settings** > **Pages**
2. Under **Build and deployment**:
   - **Source**: Select "GitHub Actions"
3. The deployment will happen automatically when code is pushed to the main branch

### Permissions

The workflow uses OIDC authentication which requires:
- `pages: write` permission (to deploy)
- `id-token: write` permission (to verify deployment origin)

Both permissions are correctly configured in the workflow.

### Troubleshooting GitHub Pages Deployment

#### Issue: Deploy job shows "action_required" or fails

**Possible Causes:**
1. GitHub Pages is not enabled or not set to use "GitHub Actions" as the source
2. The repository environment "github-pages" requires manual approval
3. The deployment needs to be approved by a repository administrator

**Solutions:**

1. **Verify GitHub Pages is configured correctly:**
   - Go to **Settings** > **Pages**
   - Ensure **Source** is set to "GitHub Actions" (not "Deploy from a branch")
   - Save if you made any changes

2. **Check environment protection rules:**
   - Go to **Settings** > **Environments** > **github-pages**
   - If "Required reviewers" is enabled, deployments will need manual approval
   - Either approve pending deployments or disable required reviewers if not needed

3. **Verify permissions:**
   - The workflow has the correct permissions configured
   - Check that GitHub Actions has permission to deploy to Pages in repository settings

4. **First deployment:**
   - The first deployment to GitHub Pages might require manual approval or configuration
   - After the first successful deployment, subsequent deployments should work automatically

5. **Re-run the workflow:**
   - Sometimes the first run fails due to environment setup
   - Try re-running the failed workflow after verifying the settings above
