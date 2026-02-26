# MSRV Policy

## Purpose

This workspace defines a Minimum Supported Rust Version (MSRV) to keep builds predictable for contributors and downstream users.

## Current Policy

- `edition = "2024"`
- `MSRV (rust-version) = "1.85"`

The workspace `rust-version` in `/Cargo.toml` is the source of truth and is inherited by all member crates.

## Compatibility Targets

- `MSRV`: `1.85` (must compile)
- `stable`: latest stable Rust (must compile and pass tests)

Optional future target:

- `beta`: compile/test informational job (non-blocking until CI matures)

## Change Management

Raise MSRV only when at least one of these is true:

- A required language/library feature needs a newer compiler
- A dependency raises its effective MSRV and pinning is not desirable
- Tooling or security maintenance requires the upgrade

When MSRV changes:

1. Update `rust-version` in workspace `Cargo.toml`
2. Mention the bump in changelog/release notes
3. Update CI matrix/jobs (if pinned)
4. Use a dedicated commit (recommended)

## Contributor Guidance

- Using a newer local toolchain is allowed
- New code should avoid unnecessary use of features that require raising MSRV
- If a newer compiler feature is beneficial, open a proposal/issue before using it across the workspace
