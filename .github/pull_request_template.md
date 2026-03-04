## Summary

Describe what changed and why.

## Validation

- [ ] `cargo check --workspace --all-targets --all-features --locked`
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -A clippy::all -A clippy::pedantic -D clippy::correctness -D clippy::suspicious`
- [ ] `cargo test -p linguasteg-cli --tests --locked`
- [ ] `./scripts/ci/smoke_e2e.sh`

## Governance

- [ ] Commit titles follow Conventional Commits
- [ ] I did not include private planning files (`task_plan.md`, `findings.md`, `progress.md`)
- [ ] If this affects contracts/interfaces, tests were updated accordingly
- [ ] I confirmed this PR is targeting the correct branch (`develop` or `main`)
