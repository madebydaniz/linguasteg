# Pre-Release Checklist

## Phase Snapshot (2026-03-04)

Current status for release preparation:

- Phase 1-5: complete (workspace foundation, runtime registry, encode/decode contracts, fa/en text decode, secure envelope flow)
- Phase 6: complete (governance, release automation, release drill, readiness gates)
- Phase 7.1: complete (`LSTEG_SECRET_FILE` support and precedence hardening)
- Phase 7.2: complete (decode hardening for non-envelope/invalid-envelope payloads)

Recommended next execution phase:

- Phase 8: release candidate on `develop` and PR to `main`

## Local Readiness Commands

Run from repository root:

- Fast gate (recommended before every release PR):
  - `./scripts/ci/pre_release_checklist.sh fast`
- Full gate (recommended before final merge to `main`):
  - `./scripts/ci/pre_release_checklist.sh full`
- Freeze go/no-go report (recommended before opening PR to `main`):
  - `./scripts/ci/go_no_go_check.sh report`
- Freeze go/no-go strict (recommended right before merge to `main`):
  - `./scripts/ci/go_no_go_check.sh strict`

What `fast` runs:

- `cargo check --workspace --all-targets --all-features --locked`
- `cargo test -p linguasteg-cli --tests --locked`
- `./scripts/ci/smoke_e2e.sh`
- `./scripts/ci/release_drill.sh preflight-only`
- `./scripts/ci/verify_repo_hygiene.sh`

What `full` adds:

- strict clippy bug-risk profile
- workspace-wide tests

## GitHub Readiness Checks (PR to `main`)

Required status checks:

- `quality`
- `contract-tests`
- `e2e-smoke`
- `release-readiness`
- `dependency-review`
- `cargo-audit`
- `Analyze (Rust)`

Additional release checks:

- `release-drill` workflow green on `develop` (manual run on latest commit is recommended)
- `publish-crates` workflow already present and wired to release `published`

## Release Execution Path

1. Ensure `develop` is green and up to date
2. Run `./scripts/ci/go_no_go_check.sh report`
3. Open PR from `develop` to `main`
4. Merge after required checks + approval
5. Let `release-please` open/update release PR on `main`
6. Merge release PR
7. Confirm GitHub Release published
8. Confirm `publish-crates` jobs (`preflight`, `dry-run`, `publish`) all green
9. Verify crates availability on crates.io
