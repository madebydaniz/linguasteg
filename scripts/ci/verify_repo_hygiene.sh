#!/usr/bin/env bash
set -euo pipefail

readonly FORBIDDEN_FILES=(
  "task_plan.md"
  "findings.md"
  "progress.md"
)

resolve_diff_range() {
  local base_ref="${1:-${GITHUB_BASE_REF:-}}"
  if [[ -n "${base_ref}" ]]; then
    git fetch --no-tags --depth=1 origin "${base_ref}" >/dev/null 2>&1
    echo "origin/${base_ref}...HEAD"
    return 0
  fi

  if git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    echo "HEAD~1..HEAD"
    return 0
  fi

  echo ""
}

main() {
  local base_ref="${1:-}"
  local range changed hit=false

  range="$(resolve_diff_range "${base_ref}")"
  if [[ -z "${range}" ]]; then
    changed="$(git status --porcelain | awk '{print $2}')"
  else
    changed="$(git diff --name-only "${range}")"
  fi

  for file in "${FORBIDDEN_FILES[@]}"; do
    if grep -Fxq "${file}" <<<"${changed}"; then
      echo "::error::forbidden planning file changed: ${file}"
      hit=true
    fi
  done

  if [[ "${hit}" == "true" ]]; then
    exit 1
  fi

  echo "repo hygiene ok: forbidden planning files are unchanged"
}

main "$@"
