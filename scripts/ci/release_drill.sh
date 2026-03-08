#!/usr/bin/env bash
set -euo pipefail

readonly RELEASE_PLEASE_CONFIG=".github/release-please-config.json"
readonly RELEASE_PLEASE_MANIFEST=".github/.release-please-manifest.json"

require_tool() {
  local tool="$1"
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "::error::required tool '${tool}' is not available"
    exit 1
  fi
}

assert_file_exists() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "::error::required file '${path}' does not exist"
    exit 1
  fi
}

workspace_crates() {
  cargo metadata --format-version 1 --no-deps \
    | jq -r '.packages[] | select((.publish == null) or ((.publish | type) == "array" and (.publish | length) > 0)) | .name' \
    | sort -u
}

release_config_crates() {
  jq -r '.packages | keys[]' "${RELEASE_PLEASE_CONFIG}" | sort -u
}

release_manifest_crates() {
  jq -r 'keys[]' "${RELEASE_PLEASE_MANIFEST}" | sort -u
}

assert_exact_match() {
  local label="$1"
  local expected_file="$2"
  local actual_file="$3"

  local missing unexpected
  missing="$(comm -23 "${expected_file}" "${actual_file}")"
  unexpected="$(comm -13 "${expected_file}" "${actual_file}")"

  if [[ -n "${missing}" ]]; then
    echo "::error::${label}: missing entries:"
    echo "${missing}"
    exit 1
  fi

  if [[ -n "${unexpected}" ]]; then
    echo "::error::${label}: unexpected entries:"
    echo "${unexpected}"
    exit 1
  fi
}

validate_release_files() {
  assert_file_exists "${RELEASE_PLEASE_CONFIG}"
  assert_file_exists "${RELEASE_PLEASE_MANIFEST}"
  jq -e . "${RELEASE_PLEASE_CONFIG}" >/dev/null
  jq -e . "${RELEASE_PLEASE_MANIFEST}" >/dev/null
}

validate_release_mapping() {
  local workspace release_cfg release_manifest
  workspace="$(mktemp)"
  release_cfg="$(mktemp)"
  release_manifest="$(mktemp)"

  workspace_crates >"${workspace}"
  release_config_crates >"${release_cfg}"
  release_manifest_crates >"${release_manifest}"

  assert_exact_match "release-please config vs workspace crates" "${workspace}" "${release_cfg}"
  assert_exact_match "release-please manifest vs workspace crates" "${workspace}" "${release_manifest}"

  rm -f "${workspace}" "${release_cfg}" "${release_manifest}"
}

workspace_version() {
  cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "linguasteg") | .version'
}

run_preflight() {
  local version
  version="$(workspace_version)"
  ./scripts/ci/publish_release.sh preflight "v${version}"
}

run_dry_run() {
  ./scripts/ci/publish_release.sh dry-run
}

main() {
  local mode="${1:-full}"
  require_tool cargo
  require_tool jq
  validate_release_files
  validate_release_mapping
  run_preflight

  case "${mode}" in
    full)
      run_dry_run
      ;;
    preflight-only)
      ;;
    *)
      echo "usage: $0 [full|preflight-only]"
      exit 2
      ;;
  esac
}

main "$@"
