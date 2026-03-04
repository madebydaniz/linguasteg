# Release Governance

## Scope

This policy defines how changes move from `develop` to `main`, and how `main` changes are released to crates.io.

## Branch Model

- `main`: protected release branch
- `develop`: integration branch for iterative delivery

All feature work should land in `develop` first, then move to `main` via pull request.

## Required Main Branch Protection

Configure a ruleset for `main` with at least:

- Require pull request before merging
- Require at least 1 approval
- Require status checks to pass
- Require branches to be up to date before merging
- Block force pushes

Required status checks:

- `quality`
- `contract-tests`
- `e2e-smoke`
- `dependency-review`
- `cargo-audit`
- `Analyze (Rust)`

## CI and Release Flow

1. PR merged into `main`
2. `release-please` runs on `main` push and opens/updates release PR
3. Release PR merge creates tags and GitHub Release
4. `publish-crates` runs on release `published`
5. Publish pipeline executes:
   - `preflight`
   - `dry-run`
   - `publish`

If `preflight` or `dry-run` fails, publishing is blocked.

## Maintainer Checklist

Before merging to `main`:

- Ensure commit messages follow Conventional Commits
- Ensure release-impacting changes include test coverage
- Ensure private working files are not included:
  - `task_plan.md`
  - `findings.md`
  - `progress.md`
- Confirm required checks are green

After release:

- Verify crates are visible on crates.io
- Verify release notes content is correct

## Hotfix Guidance

For urgent production issues:

1. Open a dedicated hotfix PR to `main`
2. Keep diff minimal and scoped
3. Require the same status checks
4. Back-merge the hotfix into `develop`
