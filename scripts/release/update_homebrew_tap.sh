#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: update_homebrew_tap.sh \
  --version <vX.Y.Z> \
  --release-tag <tag> \
  --checksums <path/to/checksums.txt> \
  --source-repo <owner/repo> \
  --tap-repo <owner/homebrew-tap> \
  --token <github-token>
USAGE
}

VERSION=""
RELEASE_TAG=""
CHECKSUMS_FILE=""
SOURCE_REPO=""
TAP_REPO=""
TOKEN=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --release-tag)
      RELEASE_TAG="$2"
      shift 2
      ;;
    --checksums)
      CHECKSUMS_FILE="$2"
      shift 2
      ;;
    --source-repo)
      SOURCE_REPO="$2"
      shift 2
      ;;
    --tap-repo)
      TAP_REPO="$2"
      shift 2
      ;;
    --token)
      TOKEN="$2"
      shift 2
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

if [[ -z "$VERSION" || -z "$CHECKSUMS_FILE" || -z "$SOURCE_REPO" || -z "$TAP_REPO" || -z "$TOKEN" ]]; then
  echo "error: missing required arguments" >&2
  usage
  exit 2
fi

if [[ -z "$RELEASE_TAG" ]]; then
  RELEASE_TAG="linguasteg-${VERSION}"
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RENDER_SCRIPT="${ROOT_DIR}/scripts/release/render_homebrew_formula.sh"
if [[ ! -x "$RENDER_SCRIPT" ]]; then
  echo "error: render script is not executable: $RENDER_SCRIPT" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

TAP_URL="https://x-access-token:${TOKEN}@github.com/${TAP_REPO}.git"
TAP_DIR="${TMP_DIR}/homebrew-tap"

git clone "$TAP_URL" "$TAP_DIR"
mkdir -p "${TAP_DIR}/Formula"

"$RENDER_SCRIPT" \
  --version "$VERSION" \
  --release-tag "$RELEASE_TAG" \
  --repo "$SOURCE_REPO" \
  --checksums "$CHECKSUMS_FILE" \
  --output "${TAP_DIR}/Formula/lsteg.rb"

pushd "$TAP_DIR" >/dev/null
if git diff --quiet -- Formula/lsteg.rb; then
  echo "homebrew formula already up to date"
  popd >/dev/null
  exit 0
fi

git add Formula/lsteg.rb
git commit -m "chore: update lsteg formula ${VERSION}"
git push origin HEAD
popd >/dev/null

echo "homebrew tap updated: ${TAP_REPO}"
