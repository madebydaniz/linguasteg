#!/usr/bin/env bash
set -euo pipefail

readonly REQUIRED_WORKFLOWS=(
  ".github/workflows/ci-main-gate.yml"
  ".github/workflows/codeql.yml"
  ".github/workflows/release-please.yml"
  ".github/workflows/publish-crates.yml"
  ".github/workflows/release-drill.yml"
)

run_step() {
  local label="$1"
  shift
  echo "==> ${label}"
  "$@"
}

assert_file_exists() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "::error::required file is missing: ${path}"
    exit 1
  fi
}

check_required_workflows() {
  local workflow
  for workflow in "${REQUIRED_WORKFLOWS[@]}"; do
    assert_file_exists "${workflow}"
  done
}

check_release_metadata_files() {
  assert_file_exists ".github/release-please-config.json"
  assert_file_exists ".github/.release-please-manifest.json"
}

check_freeze_git_state_strict() {
  local status
  status="$(git status --porcelain)"
  if [[ -n "${status}" ]]; then
    echo "::error::strict go/no-go requires clean git working tree"
    echo "${status}"
    exit 1
  fi
}

mode_report() {
  run_step "Pre-release fast checklist" ./scripts/ci/pre_release_checklist.sh fast
  run_step "Required workflow files" check_required_workflows
  run_step "Release metadata files" check_release_metadata_files
  echo "GO/NO-GO report checks passed"
}

mode_strict() {
  mode_report
  run_step "Strict freeze git-state check" check_freeze_git_state_strict
  echo "GO decision gates passed (strict)"
}

main() {
  local mode="${1:-report}"
  case "${mode}" in
    report)
      mode_report
      ;;
    strict)
      mode_strict
      ;;
    *)
      echo "usage: $0 [report|strict]"
      exit 2
      ;;
  esac
}

main "$@"
