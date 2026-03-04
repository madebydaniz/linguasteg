# Go/No-Go Checklist

## Purpose

This checklist is the final release-freeze gate before opening or merging a PR from `develop` to `main`.

## Runbook

Run from repository root:

- Report mode:
  - `./scripts/ci/go_no_go_check.sh report`
- Strict mode:
  - `./scripts/ci/go_no_go_check.sh strict`

Mode behavior:

- `report`: validates release readiness artifacts and fast technical gates
- `strict`: includes `report` plus clean working tree requirement

## What Is Validated

`go_no_go_check.sh report` validates:

- `./scripts/ci/pre_release_checklist.sh fast` passes
- Required workflow files exist:
  - `.github/workflows/ci-main-gate.yml`
  - `.github/workflows/codeql.yml`
  - `.github/workflows/release-please.yml`
  - `.github/workflows/publish-crates.yml`
  - `.github/workflows/release-drill.yml`
- Release metadata files exist:
  - `release-please-config.json`
  - `.release-please-manifest.json`

`go_no_go_check.sh strict` also validates:

- `git status --porcelain` is empty

## Decision Rule

- `GO`: all checks pass
- `NO-GO`: any check fails

If `NO-GO`, fix the failing item and rerun the same mode before proceeding.
