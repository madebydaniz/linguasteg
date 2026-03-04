#!/usr/bin/env bash
set -euo pipefail

export LSTEG_SECRET="${LSTEG_SECRET:-linguasteg-ci-smoke-secret}"

encode_json="$(cargo run --locked -q -p linguasteg-cli -- encode --message "smoke-salam" --format json)"
if [[ "${encode_json}" != *"\"mode\":\"proto-encode\""* ]]; then
  echo "smoke failure: encode json missing mode"
  exit 1
fi

decode_json="$(
  printf "%s" "${encode_json}" \
    | cargo run --locked -q -p linguasteg-cli -- decode --format json
)"
if [[ "${decode_json}" != *"\"payload_utf8\":\"smoke-salam\""* ]]; then
  echo "smoke failure: decode json missing expected payload"
  exit 1
fi

analyze_json="$(
  printf "%s" "${encode_json}" \
    | cargo run --locked -q -p linguasteg-cli -- analyze --format json
)"
if [[ "${analyze_json}" != *"\"integrity_ok\":true"* ]]; then
  echo "smoke failure: analyze json integrity is not true"
  exit 1
fi

echo "smoke e2e passed"
