#!/usr/bin/env bash
set -euo pipefail

run_step() {
  local label="$1"
  shift
  echo "==> ${label}"
  "$@"
}

run_fast() {
  run_step "Cargo check" cargo check --workspace --all-targets --all-features --locked
  run_step "Contract tests (cli)" cargo test -p linguasteg-cli --tests --locked
  run_step "E2E smoke" ./scripts/ci/smoke_e2e.sh
  run_step "Release preflight drill" ./scripts/ci/release_drill.sh preflight-only
  run_step "Repository hygiene guard" ./scripts/ci/verify_repo_hygiene.sh
}

run_full() {
  run_fast
  run_step \
    "Clippy (bug-risk lints)" \
    cargo clippy --workspace --all-targets --all-features --locked -- -A clippy::all -A clippy::pedantic -D clippy::correctness -D clippy::suspicious
  run_step "Workspace tests" cargo test --workspace --all-targets --locked
}

main() {
  local mode="${1:-fast}"
  case "${mode}" in
    fast)
      run_fast
      ;;
    full)
      run_full
      ;;
    *)
      echo "usage: $0 [fast|full]"
      exit 2
      ;;
  esac
}

main "$@"
