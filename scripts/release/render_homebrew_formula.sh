#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: render_homebrew_formula.sh \
  --version <vX.Y.Z> \
  --repo <owner/repo> \
  --checksums <path/to/checksums.txt> \
  --output <path/to/lsteg.rb>
USAGE
}

VERSION=""
REPO=""
CHECKSUMS_FILE=""
OUTPUT_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --repo)
      REPO="$2"
      shift 2
      ;;
    --checksums)
      CHECKSUMS_FILE="$2"
      shift 2
      ;;
    --output)
      OUTPUT_FILE="$2"
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

if [[ -z "$VERSION" || -z "$REPO" || -z "$CHECKSUMS_FILE" || -z "$OUTPUT_FILE" ]]; then
  echo "error: missing required arguments" >&2
  usage
  exit 2
fi

if [[ "$VERSION" != v* ]]; then
  VERSION="v${VERSION}"
fi

if [[ ! -f "$CHECKSUMS_FILE" ]]; then
  echo "error: checksums file not found: $CHECKSUMS_FILE" >&2
  exit 1
fi

asset_name() {
  local target="$1"
  printf 'lsteg-%s-%s.tar.gz\n' "$VERSION" "$target"
}

checksum_for() {
  local asset="$1"
  local value
  value="$(grep "  ${asset}$" "$CHECKSUMS_FILE" | awk '{print $1}')"
  if [[ -z "$value" ]]; then
    echo "error: checksum not found for asset '$asset'" >&2
    exit 1
  fi
  printf '%s\n' "$value"
}

A_MAC_ARM="$(asset_name aarch64-apple-darwin)"
A_MAC_X64="$(asset_name x86_64-apple-darwin)"
A_LINUX_X64="$(asset_name x86_64-unknown-linux-gnu)"

S_MAC_ARM="$(checksum_for "$A_MAC_ARM")"
S_MAC_X64="$(checksum_for "$A_MAC_X64")"
S_LINUX_X64="$(checksum_for "$A_LINUX_X64")"

VERSION_CLEAN="${VERSION#v}"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

cat > "$OUTPUT_FILE" <<FORMULA
class Lsteg < Formula
  desc "Multilingual linguistic steganography CLI"
  homepage "https://github.com/${REPO}"
  version "${VERSION_CLEAN}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "${BASE_URL}/${A_MAC_ARM}"
      sha256 "${S_MAC_ARM}"
    else
      url "${BASE_URL}/${A_MAC_X64}"
      sha256 "${S_MAC_X64}"
    end
  end

  on_linux do
    url "${BASE_URL}/${A_LINUX_X64}"
    sha256 "${S_LINUX_X64}"
  end

  def install
    bin.install "lsteg"
  end

  test do
    assert_match "LinguaSteg CLI", shell_output("#{bin}/lsteg --help")
  end
end
FORMULA
