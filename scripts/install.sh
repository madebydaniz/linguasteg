#!/usr/bin/env bash
set -euo pipefail

OWNER="madebydaniz"
REPO="linguasteg"
BINARY_NAME="lsteg"
WORKFLOW_IDENTITY_REGEX='^https://github.com/madebydaniz/linguasteg/\.github/workflows/release-binaries\.yml@refs/(heads/main|tags/.+)$'
OIDC_ISSUER="https://token.actions.githubusercontent.com"

VERSION=""
INSTALL_DIR=""
VERIFY_SIGNATURE="true"

usage() {
  cat <<USAGE
Usage: install.sh [options]

Options:
  --version <linguasteg-vX.Y.Z|vX.Y.Z|X.Y.Z>  Install a specific release (default: latest)
  --install-dir <path>      Target bin directory (default: ~/.local/bin or /usr/local/bin)
  --no-verify-signature     Skip cosign signature verification (not recommended)
  -h, --help                Show help
USAGE
}

require_tool() {
  local tool="$1"
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: required tool '$tool' was not found" >&2
    exit 1
  fi
}

resolve_latest_tag() {
  local api_url="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
  local response
  response="$(curl -fsSL "$api_url")"
  local tag
  tag="$(printf '%s' "$response" | grep -m1 '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')"
  if [[ -z "$tag" ]]; then
    echo "error: failed to resolve latest release tag" >&2
    exit 1
  fi
  printf '%s\n' "$tag"
}

normalize_release_refs() {
  local raw="$1"

  if [[ "$raw" =~ ^linguasteg-v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    VERSION="v${BASH_REMATCH[1]}"
    RELEASE_TAG="$raw"
    return
  fi

  if [[ "$raw" =~ ^v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    VERSION="$raw"
    RELEASE_TAG="linguasteg-${raw}"
    return
  fi

  if [[ "$raw" =~ ^([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    VERSION="v${raw}"
    RELEASE_TAG="linguasteg-v${raw}"
    return
  fi

  echo "error: unsupported version/tag '${raw}'" >&2
  echo "expected one of: linguasteg-vX.Y.Z, vX.Y.Z, or X.Y.Z" >&2
  exit 2
}

resolve_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64) printf '%s\n' "aarch64-apple-darwin" ;;
        x86_64) printf '%s\n' "x86_64-apple-darwin" ;;
        *) echo "error: unsupported macOS architecture '$arch'" >&2; exit 1 ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64|amd64) printf '%s\n' "x86_64-unknown-linux-gnu" ;;
        *) echo "error: unsupported Linux architecture '$arch'" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "error: unsupported OS '$os'" >&2
      exit 1
      ;;
  esac
}

sha256_file() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  else
    shasum -a 256 "$file" | awk '{print $1}'
  fi
}

verify_checksum() {
  local checksum_file="$1"
  local asset_file="$2"
  local asset_name="$3"

  local expected actual
  expected="$(grep "  ${asset_name}$" "$checksum_file" | awk '{print $1}' || true)"

  if [[ -z "$expected" ]]; then
    echo "error: checksum entry for '${asset_name}' not found" >&2
    exit 1
  fi

  actual="$(sha256_file "$asset_file")"
  if [[ "$expected" != "$actual" ]]; then
    echo "error: checksum mismatch for ${asset_name}" >&2
    echo "expected: $expected" >&2
    echo "actual:   $actual" >&2
    exit 1
  fi
}

select_install_dir() {
  if [[ -n "$INSTALL_DIR" ]]; then
    printf '%s\n' "$INSTALL_DIR"
    return
  fi

  if [[ -w "/usr/local/bin" ]]; then
    printf '%s\n' "/usr/local/bin"
    return
  fi

  printf '%s\n' "${HOME}/.local/bin"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --install-dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    --no-verify-signature)
      VERIFY_SIGNATURE="false"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument '$1'" >&2
      usage
      exit 2
      ;;
  esac
done

require_tool curl
require_tool tar

if [[ -z "$VERSION" ]]; then
  normalize_release_refs "$(resolve_latest_tag)"
else
  normalize_release_refs "$VERSION"
fi

TARGET="$(resolve_target)"
ASSET_NAME="${BINARY_NAME}-${VERSION}-${TARGET}.tar.gz"
RELEASE_URL="https://github.com/${OWNER}/${REPO}/releases/download/${RELEASE_TAG}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

CHECKSUMS_PATH="${TMP_DIR}/checksums.txt"
SIGNATURE_PATH="${TMP_DIR}/checksums.txt.sig"
CERT_PATH="${TMP_DIR}/checksums.txt.pem"
ASSET_PATH="${TMP_DIR}/${ASSET_NAME}"

curl -fsSL "${RELEASE_URL}/checksums.txt" -o "$CHECKSUMS_PATH"
curl -fsSL "${RELEASE_URL}/checksums.txt.sig" -o "$SIGNATURE_PATH"
curl -fsSL "${RELEASE_URL}/checksums.txt.pem" -o "$CERT_PATH"
curl -fsSL "${RELEASE_URL}/${ASSET_NAME}" -o "$ASSET_PATH"

if [[ "$VERIFY_SIGNATURE" == "true" ]]; then
  require_tool cosign
  cosign verify-blob \
    --certificate "$CERT_PATH" \
    --signature "$SIGNATURE_PATH" \
    --certificate-identity-regexp "$WORKFLOW_IDENTITY_REGEX" \
    --certificate-oidc-issuer "$OIDC_ISSUER" \
    "$CHECKSUMS_PATH" >/dev/null
fi

verify_checksum "$CHECKSUMS_PATH" "$ASSET_PATH" "$ASSET_NAME"

tar -xzf "$ASSET_PATH" -C "$TMP_DIR"

if [[ ! -f "${TMP_DIR}/${BINARY_NAME}" ]]; then
  echo "error: release archive does not contain '${BINARY_NAME}'" >&2
  exit 1
fi

TARGET_DIR="$(select_install_dir)"
mkdir -p "$TARGET_DIR"

if [[ -w "$TARGET_DIR" ]]; then
  install -m 0755 "${TMP_DIR}/${BINARY_NAME}" "${TARGET_DIR}/${BINARY_NAME}"
else
  if command -v sudo >/dev/null 2>&1; then
    sudo install -m 0755 "${TMP_DIR}/${BINARY_NAME}" "${TARGET_DIR}/${BINARY_NAME}"
  else
    echo "error: '${TARGET_DIR}' is not writable and sudo is unavailable" >&2
    exit 1
  fi
fi

echo "installed: ${TARGET_DIR}/${BINARY_NAME}"
"${TARGET_DIR}/${BINARY_NAME}" --help >/dev/null
echo "verify: ${BINARY_NAME} --help ok"
