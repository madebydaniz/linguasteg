#!/usr/bin/env bash
set -euo pipefail

readonly CRATES=(
  "linguasteg-core"
  "linguasteg-models"
  "linguasteg-eval"
  "linguasteg"
  "linguasteg-cli"
)

metadata=""

require_tool() {
  local tool="$1"
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "::error::required tool '${tool}' is not available"
    exit 1
  fi
}

load_metadata() {
  metadata="$(cargo metadata --format-version 1 --no-deps)"
}

crate_version() {
  local crate_name="$1"
  echo "${metadata}" | jq -r --arg name "${crate_name}" '.packages[] | select(.name == $name) | .version'
}

crate_license() {
  local crate_name="$1"
  echo "${metadata}" | jq -r --arg name "${crate_name}" '.packages[] | select(.name == $name) | .license'
}

crate_repository() {
  local crate_name="$1"
  echo "${metadata}" | jq -r --arg name "${crate_name}" '.packages[] | select(.name == $name) | .repository'
}

assert_release_tag_matches_workspace_version() {
  local release_tag="${1:-}"
  if [[ -z "${release_tag}" ]]; then
    return 0
  fi

  local normalized=""
  if [[ "${release_tag}" =~ ^v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    normalized="${BASH_REMATCH[1]}"
  elif [[ "${release_tag}" =~ ^linguasteg-v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    normalized="${BASH_REMATCH[1]}"
  else
    echo "::error::release tag '${release_tag}' must match v<major>.<minor>.<patch> or linguasteg-v<major>.<minor>.<patch>"
    exit 1
  fi

  local expected
  expected="$(echo "${metadata}" | jq -r '.packages[] | select(.name == "linguasteg") | .version')"

  if [[ "${expected}" != "${normalized}" ]]; then
    echo "::error::release tag '${release_tag}' does not match workspace version '${expected}'"
    exit 1
  fi
}

crate_exists() {
  local crate_name="$1"
  local version="$2"
  local status
  status="$(curl -s -o /dev/null -w "%{http_code}" "https://crates.io/api/v1/crates/${crate_name}/${version}")"
  [[ "${status}" == "200" ]]
}

publish_with_retry() {
  local crate_name="$1"
  local attempts=5
  local delay=15

  for try in $(seq 1 "${attempts}"); do
    if cargo publish -p "${crate_name}" --locked; then
      return 0
    fi

    if [[ "${try}" -eq "${attempts}" ]]; then
      echo "::error::failed to publish ${crate_name} after ${attempts} attempts"
      return 1
    fi

    sleep "$((delay * try))"
  done
}

run_preflight() {
  local release_tag="${1:-}"
  require_tool jq
  load_metadata
  assert_release_tag_matches_workspace_version "${release_tag}"

  for crate in "${CRATES[@]}"; do
    local version license repository
    version="$(crate_version "${crate}")"
    license="$(crate_license "${crate}")"
    repository="$(crate_repository "${crate}")"

    if [[ -z "${version}" || "${version}" == "null" ]]; then
      echo "::error::could not detect version for ${crate}"
      exit 1
    fi
    if [[ -z "${license}" || "${license}" == "null" ]]; then
      echo "::error::crate ${crate}@${version} is missing license metadata"
      exit 1
    fi
    if [[ -z "${repository}" || "${repository}" == "null" ]]; then
      echo "::error::crate ${crate}@${version} is missing repository metadata"
      exit 1
    fi

    echo "preflight ok: ${crate}@${version}"
  done
}

run_dry_run() {
  require_tool jq
  load_metadata
  unset CARGO_REGISTRY_TOKEN || true

  for crate in "${CRATES[@]}"; do
    local version
    version="$(crate_version "${crate}")"
    echo "dry-run publish ${crate}@${version}"
    cargo publish -p "${crate}" --dry-run --locked --no-verify
  done
}

run_publish() {
  require_tool jq
  require_tool curl
  load_metadata

  if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    echo "::error::CRATES_IO_TOKEN is not configured"
    exit 1
  fi

  for crate in "${CRATES[@]}"; do
    local version
    version="$(crate_version "${crate}")"

    if crate_exists "${crate}" "${version}"; then
      echo "${crate}@${version} already published, skipping"
      continue
    fi

    echo "publishing ${crate}@${version}"
    publish_with_retry "${crate}"
  done
}

main() {
  local command="${1:-}"
  local release_tag="${2:-}"

  case "${command}" in
    preflight)
      run_preflight "${release_tag}"
      ;;
    dry-run)
      run_dry_run
      ;;
    publish)
      run_publish
      ;;
    *)
      echo "usage: $0 <preflight|dry-run|publish> [release-tag]"
      exit 2
      ;;
  esac
}

main "$@"
